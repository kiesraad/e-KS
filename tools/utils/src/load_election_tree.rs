#![allow(clippy::too_many_lines, clippy::cognitive_complexity)]

use std::path::Path;

use eml_nl::{
    documents::master_election_tree::{MasterElectionTree, MetRegion},
    io::EMLRead,
    utils::RegionCategory,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::slugify_teletex;

/// Convert a region name to a valid PascalCase Rust identifier
fn to_ident(name: &str) -> String {
    slugify_teletex(name, false)
        .split('-')
        .filter(|s| !s.is_empty())
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let mut out: String = first.to_uppercase().collect();
                    out.push_str(chars.as_str());
                    out
                }
            }
        })
        .collect()
}

/// RegionNumber as u16, defaulting to 0 if absent
fn num(r: &MetRegion) -> u16 {
    r.key.number.unwrap_or(0)
}

/// RegionNumber of the "NBSB" pseudo-municipality (Nationaal Brief Stembureau),
/// It only applies to TK and EP elections, which we don't generate districts for
/// yet, so it is filtered out of every municipality we generate here.
const NBSB_REGION_NUMBER: u16 = 9010;

/// Regions from MasterElectionTree, sorted by category and election type
#[derive(Default)]
struct Districts {
    /// EK/PS: PROVINCIE + KIESCOLLEGE
    provinces_and_colleges: Vec<MetRegion>,
    /// EK: PROVINCIAAL_STEMBUREAU; tuple is (region, parent EK number)
    province_polling_stations: Vec<(MetRegion, u16)>,
    /// TK: KIESKRING
    electoral_districts: Vec<MetRegion>,
    /// PS: PROVINCIAAL_KIESKRING; tuple is (region, parent PROVINCIE number)
    province_electoral_district: Vec<(MetRegion, u16)>,
    /// TK/PS: GEMEENTE; tuple is (region, parent PROVINCIAAL_KIESKRING number, grandparent PROVINCIE number)
    municipality: Vec<(MetRegion, u16, u16)>,
    /// WS: WATERSCHAP
    water_authority: Vec<MetRegion>,
    /// WS: WATERSCHAP_KIESKRING; tuple is (region, parent WATERSCHAP number)
    water_authority_electoral_district: Vec<(MetRegion, u16)>,
    /// WS: WATERSCHAP_GEMEENTE references; tuple is (WATERSCHAP_GEMEENTE region, base Gm ident, parent WATERSCHAP number, FrysianExportAllowed)
    /// When FrysianExportAllowed differs from the TK/PS gemeente, a separate Fr-suffixed variant is generated
    water_authority_municipality: Vec<(MetRegion, String, u16, bool)>,
}

impl Districts {
    pub fn parse_from_file(file: &Path) -> Self {
        let xml = std::fs::read_to_string(file).expect("Could not read MasterElectionTree.xml");
        let tree = MasterElectionTree::parse_eml(&xml, eml_nl::io::EMLParsingMode::Strict)
            .expect("Failed to parse MasterElectionTree.xml");

        Districts::from(tree)
    }
}

/// One ElectoralDistrict variant with its precomputed names
struct RegionEntry<'a> {
    /// Rust enum variant name, e.g. "SbGroningen", "GmLeeuwarden"
    variant: String,
    /// Stable typed code for use in e.g. the EML API, e.g. "sb2", "gm358"
    code: String,
    region: &'a MetRegion,
}

impl Districts {
    /// All districts in enum declaration order, with precomputed variant name and code
    fn all_entries<'a>(&'a self) -> Vec<RegionEntry<'a>> {
        let mut entries = Vec::new();
        for r in &self.provinces_and_colleges {
            let code_prefix = match r.key.category {
                RegionCategory::Province => "prov",
                RegionCategory::ElectoralCollege => "kc",
                _ => panic!("provinces_and_colleges should only contain provinces and colleges"),
            };
            entries.push(RegionEntry {
                variant: to_ident(&r.name),
                code: format!("{}{}", code_prefix, num(r)),
                region: r,
            });
        }
        for (r, _) in &self.province_polling_stations {
            entries.push(RegionEntry {
                variant: format!("Sb{}", to_ident(&r.name)),
                code: format!("sb{}", num(r)),
                region: r,
            });
        }
        for r in &self.electoral_districts {
            entries.push(RegionEntry {
                variant: format!("Tk{}", to_ident(&r.name)),
                code: format!("tk{}", num(r)),
                region: r,
            });
        }
        for (r, _) in &self.province_electoral_district {
            entries.push(RegionEntry {
                variant: format!("Ps{}", to_ident(&r.name)),
                code: format!("pk{}", num(r)),
                region: r,
            });
        }
        // The "Fr" suffix indicates a municipality has FrysianExportAllowed true.
        // Some municipalities appear both in Fryslan and in another water council,
        // so we use the suffix to remember when Frisian export is allowed.
        for (r, _, _) in &self.municipality {
            let suffix = if r.frysian_export_allowed { "Fr" } else { "" };
            entries.push(RegionEntry {
                variant: format!("Gm{}{}", to_ident(&r.name), suffix),
                code: format!("gm{}{}", num(r), suffix.to_lowercase()),
                region: r,
            });
        }
        for (r, ws_num) in &self.water_authority_electoral_district {
            entries.push(RegionEntry {
                variant: format!("Ws{}", to_ident(&r.name)),
                code: format!("ws{}", ws_num),
                region: r,
            });
        }
        // WS gemeente variants that are not already covered by PS
        let existing: std::collections::HashSet<String> =
            entries.iter().map(|e| e.variant.clone()).collect();
        let mut ws_extras: std::collections::HashSet<String> = std::collections::HashSet::new();
        for (gm_region, gm_ident, _, frisian_flag) in &self.water_authority_municipality {
            let suffix = if *frisian_flag { "Fr" } else { "" };
            let variant = format!("{}{}", gm_ident, suffix);
            if !existing.contains(&variant) && ws_extras.insert(variant.clone()) {
                entries.push(RegionEntry {
                    variant,
                    code: format!("gm{}{}", num(gm_region), suffix.to_lowercase()),
                    region: gm_region,
                });
            }
        }
        entries
    }
}

impl From<MasterElectionTree> for Districts {
    fn from(tree: MasterElectionTree) -> Self {
        let staat = &tree.root;
        let mut d = Districts::default();

        // Filter helper: iterate child regions of a given category
        let of = |cat: RegionCategory| move |r: &&MetRegion| r.key.category == cat;

        // PROVINCIE: EK provinces with their nested structure
        for prov in staat.subregions.iter().filter(of(RegionCategory::Province)) {
            let prov_num = num(prov);
            d.provinces_and_colleges.push(prov.clone());
            for sb in prov
                .subregions
                .iter()
                .filter(of(RegionCategory::ProvincePollingStation))
            {
                d.province_polling_stations.push((sb.clone(), prov_num));
                for ks in sb
                    .subregions
                    .iter()
                    .filter(of(RegionCategory::ElectoralDistrict))
                {
                    d.electoral_districts.push(ks.clone());
                    for pk in ks
                        .subregions
                        .iter()
                        .filter(of(RegionCategory::ProvinceElectoralDistrict))
                    {
                        let pk_num = num(pk);
                        d.province_electoral_district.push((pk.clone(), prov_num));
                        for gm in pk
                            .subregions
                            .iter()
                            .filter(of(RegionCategory::Municipality))
                            .filter(|r| num(r) != NBSB_REGION_NUMBER)
                        {
                            d.municipality.push((gm.clone(), pk_num, prov_num));
                        }
                    }
                }
            }
        }

        // KIESCOLLEGE: EK electoral colleges with their STEMBUREAU
        for kc in staat
            .subregions
            .iter()
            .filter(of(RegionCategory::ElectoralCollege))
        {
            let kc_num = num(kc);
            d.provinces_and_colleges.push(kc.clone());
            for sb in kc
                .subregions
                .iter()
                .filter(of(RegionCategory::ProvincePollingStation))
            {
                d.province_polling_stations.push((sb.clone(), kc_num));
            }
        }

        // KIESKRING directly under STAAT (Bonaire special case)
        for ks in staat
            .subregions
            .iter()
            .filter(of(RegionCategory::ElectoralDistrict))
        {
            d.electoral_districts.push(ks.clone());
        }

        // WATERSCHAP: WS water authorities with their municipalities
        // Build a CBS-code -> Gm ident lookup from already-collected PS municipalities
        let cbs_to_gm_ident: std::collections::HashMap<u16, String> = d
            .municipality
            .iter()
            .map(|(g, _, _)| (num(g), format!("Gm{}", to_ident(&g.name))))
            .collect();
        for ws in staat
            .subregions
            .iter()
            .filter(of(RegionCategory::WaterAuthority))
        {
            let ws_num = num(ws);
            d.water_authority.push(ws.clone());
            for wsk in ws
                .subregions
                .iter()
                .filter(of(RegionCategory::WaterAuthorityElectoralDistrict))
            {
                d.water_authority_electoral_district
                    .push((wsk.clone(), ws_num));
                for gm in wsk
                    .subregions
                    .iter()
                    .filter(of(RegionCategory::WaterAuthorityMunicipality))
                {
                    if let Some(gm_ident) = cbs_to_gm_ident.get(&num(gm)) {
                        d.water_authority_municipality.push((
                            gm.clone(),
                            gm_ident.clone(),
                            ws_num,
                            gm.frysian_export_allowed,
                        ));
                    }
                }
            }
        }

        // Sort every collection except for municipalities by its RegionNumber,
        // so ordering doesn't depend on the document order of MasterElectionTree.xml
        d.provinces_and_colleges.sort_by_key(num);
        d.province_polling_stations.sort_by_key(|(r, _)| num(r));
        d.electoral_districts.sort_by_key(num);
        d.province_electoral_district.sort_by_key(|(r, _)| num(r));
        d.water_authority.sort_by_key(num);
        d.water_authority_electoral_district
            .sort_by_key(|(r, _)| num(r));

        d
    }
}

pub fn generate_election_tree(out_dir: &Path) {
    println!("cargo:rerun-if-changed=MasterElectionTree.xml");
    let districts = Districts::parse_from_file(Path::new("MasterElectionTree.xml"));

    write_districts_file(out_dir, &districts);
    write_provinces_file(out_dir, &districts);
    write_water_councils_file(out_dir, &districts);
}

fn write_districts_file(out_dir: &Path, districts: &Districts) {
    let all = districts.all_entries();

    let variants: Vec<_> = all.iter().map(|e| format_ident!("{}", e.variant)).collect();

    let title_arms = all.iter().map(|e| {
        let id = format_ident!("{}", e.variant);
        let title = e.region.name.as_ref();
        quote! { Self::#id => #title, }
    });

    let code_arms = all.iter().map(|e| {
        let id = format_ident!("{}", e.variant);
        let code = e.code.as_str();
        quote! { Self::#id => #code, }
    });

    let region_number_arms = all.iter().map(|e| {
        let id = format_ident!("{}", e.variant);
        let number = num(e.region);
        quote! { Self::#id => #number, }
    });

    let committee_arms = all.iter().map(|e| {
        let id = format_ident!("{}", e.variant);

        // Group this region's committees by election category, preserving the
        // order they appear in MasterElectionTree.xml.
        let mut by_category: Vec<(String, Vec<TokenStream>)> = Vec::new();
        for c in &e.region.committees {
            let committee_category = format_ident!("{}", format!("{:?}", c.committee.category));
            let mut committee = quote! {
                eml_nl::common::Committee::new(eml_nl::utils::CommitteeCategory::#committee_category)
            };
            if let Some(name) = c.committee.name.as_deref() {
                committee = quote! { #committee.with_name(#name) };
            }
            if let Some(accept) = c.committee.accept_central_submissions {
                committee = quote! { #committee.with_accept_central_submissions(#accept) };
            }

            let election_category_name = format!("{:?}", c.election_category);
            match by_category
                .iter_mut()
                .find(|(name, _)| *name == election_category_name)
            {
                Some((_, committees)) => committees.push(committee),
                None => by_category.push((election_category_name, vec![committee])),
            }
        }

        if by_category.is_empty() {
            return quote! { Self::#id => Vec::new(), };
        }

        let category_arms = by_category.into_iter().map(|(election_category, committees)| {
            let election_category = format_ident!("{election_category}");
            quote! {
                eml_nl::utils::ElectionCategory::#election_category => vec![#(#committees),*],
            }
        });

        quote! {
            Self::#id => match election_category {
                #(#category_arms)*
                _ => Vec::new(),
            },
        }
    });

    let ek_variants: Vec<_> = districts
        .provinces_and_colleges
        .iter()
        .map(|r| format_ident!("{}", to_ident(&r.name)))
        .collect();

    let frisian: Vec<_> = all
        .iter()
        .filter(|e| e.region.frysian_export_allowed)
        .map(|e| {
            let id = format_ident!("{}", e.variant);
            quote! { Self::#id }
        })
        .collect();
    let frisian_body = if frisian.is_empty() {
        quote! { false }
    } else {
        quote! { matches!(self, #(#frisian)|*) }
    };

    let roman: Vec<_> = all
        .iter()
        .filter(|e| e.region.roman_numerals)
        .map(|e| {
            let id = format_ident!("{}", e.variant);
            quote! { Self::#id }
        })
        .collect();
    let roman_body = if roman.is_empty() {
        quote! { false }
    } else {
        quote! { matches!(self, #(#roman)|*) }
    };

    // sub_districts(): child districts in declaration order
    // - EK districts return their single STEMBUREAU
    // - PS and WS kieskringen return their municipalities
    let mut sub_arms: Vec<_> = Vec::new();
    for r in &districts.provinces_and_colleges {
        let r_id = format_ident!("{}", to_ident(&r.name));
        let (sb_r, _) = districts
            .province_polling_stations
            .iter()
            .find(|(_, pn)| *pn == num(r))
            .unwrap_or_else(|| panic!("No polling station for EK district {}", to_ident(&r.name)));
        let sb_id = format_ident!("Sb{}", to_ident(&sb_r.name));
        sub_arms.push(quote! { Self::#r_id => &[ElectoralDistrict::#sb_id], });
    }
    for (ps_r, ps_prov_num) in &districts.province_electoral_district {
        let ps_id = format_ident!("Ps{}", to_ident(&ps_r.name));
        let ps_pk_num = num(ps_r);
        let gm_ids: Vec<_> = districts
            .municipality
            .iter()
            .filter(|(_, pk, prov)| *pk == ps_pk_num && *prov == *ps_prov_num)
            .map(|(g, _, _)| {
                let suffix = if g.frysian_export_allowed { "Fr" } else { "" };
                format_ident!("Gm{}{}", to_ident(&g.name), suffix)
            })
            .collect();
        sub_arms.push(quote! { Self::#ps_id => &[#(ElectoralDistrict::#gm_ids),*], });
    }
    for (wsk_r, ws_num) in &districts.water_authority_electoral_district {
        let wsk_id = format_ident!("Ws{}", to_ident(&wsk_r.name));
        let gm_ids: Vec<_> = districts
            .water_authority_municipality
            .iter()
            .filter(|(_, _, wn, _)| *wn == *ws_num)
            .map(|(_, gm_id, _, frisian)| {
                if *frisian {
                    format_ident!("{}Fr", gm_id)
                } else {
                    format_ident!("{}", gm_id)
                }
            })
            .collect();
        sub_arms.push(quote! { Self::#wsk_id => &[#(ElectoralDistrict::#gm_ids),*], });
    }

    let tokens = quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
        pub enum ElectoralDistrict {
            #(#variants,)*
        }

        #[allow(clippy::too_many_lines)]
        impl ElectoralDistrict {
            pub fn title(&self) -> &'static str {
                match self { #(#title_arms)* }
            }

            pub fn code(&self) -> &'static str {
                match self { #(#code_arms)* }
            }

            pub fn region_number(&self) -> u16 {
                match self { #(#region_number_arms)* }
            }

            /// The committees active in this region for the given election category,
            /// as defined by the Kiesraad master election tree.
            #[allow(clippy::match_same_arms, clippy::cognitive_complexity)]
            pub fn committees(
                &self,
                election_category: eml_nl::utils::ElectionCategory,
            ) -> Vec<eml_nl::common::Committee> {
                match self { #(#committee_arms)* }
            }

            pub fn ek_districts() -> &'static [Self] {
                &[#(Self::#ek_variants,)*]
            }

            pub fn frisian_export_allowed(&self) -> bool {
                #frisian_body
            }

            pub fn roman_numerals(&self) -> bool {
                #roman_body
            }

            pub fn sub_districts(&self) -> &'static [ElectoralDistrict] {
                match self {
                    #(#sub_arms)*
                    _ => &[],
                }
            }
        }
    };

    write_file(tokens, "districts_generated.rs", out_dir);
}

fn write_provinces_file(out_dir: &Path, districts: &Districts) {
    // filter out KIESCOLLEGEs and keep only PROVINCIEs
    let raw_provinces: Vec<&MetRegion> = districts
        .provinces_and_colleges
        .iter()
        .filter(|r| r.key.category == RegionCategory::Province)
        .collect();

    let variants: Vec<_> = raw_provinces
        .iter()
        .map(|p| format_ident!("{}", to_ident(&p.name)))
        .collect();

    let code_arms = raw_provinces.iter().map(|p| {
        let id = format_ident!("{}", to_ident(&p.name));
        let code = format!("prov{}", num(p));
        quote! { Province::#id => #code, }
    });

    let title_arms = raw_provinces.iter().map(|p| {
        let id = format_ident!("{}", to_ident(&p.name));
        let title = p.name.as_ref();
        quote! { Province::#id => #title, }
    });

    let region_number_arms = raw_provinces.iter().map(|p| {
        let id = format_ident!("{}", to_ident(&p.name));
        let number = num(p);
        quote! { Province::#id => #number, }
    });

    let frisian_matches: Vec<_> = raw_provinces
        .iter()
        .filter(|p| p.frysian_export_allowed)
        .map(|p| {
            let id = format_ident!("{}", to_ident(&p.name));
            quote! { Province::#id }
        })
        .collect();
    let frisian_body = if frisian_matches.is_empty() {
        quote! { false }
    } else {
        quote! { matches!(self, #(#frisian_matches)|*) }
    };

    let sb_arms = raw_provinces.iter().map(|p| {
        let p_id = format_ident!("{}", to_ident(&p.name));
        let (sb_r, _) = districts
            .province_polling_stations
            .iter()
            .find(|(_, pn)| *pn == num(p))
            .unwrap_or_else(|| panic!("No polling station for province {}", to_ident(&p.name)));
        let sb_id = format_ident!("Sb{}", to_ident(&sb_r.name));
        quote! { Province::#p_id => ElectoralDistrict::#sb_id, }
    });

    let ps_arms = raw_provinces.iter().map(|p| {
        let p_id = format_ident!("{}", to_ident(&p.name));
        let p_num = num(p);
        let ps_ids: Vec<_> = districts
            .province_electoral_district
            .iter()
            .filter(|(_, pn)| *pn == p_num)
            .map(|(r, _)| format_ident!("Ps{}", to_ident(&r.name)))
            .collect();
        quote! { Province::#p_id => &[#(ElectoralDistrict::#ps_ids),*], }
    });

    let tokens = quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
        pub enum Province {
            #(#variants,)*
        }

        impl Province {
            pub const ALL: &[Province] = &[#(Province::#variants,)*];

            pub fn code(&self) -> &'static str {
                match self { #(#code_arms)* }
            }

            pub fn title(&self) -> &'static str {
                match self { #(#title_arms)* }
            }

            pub fn region_number(&self) -> u16 {
                match self { #(#region_number_arms)* }
            }

            pub fn frisian_export_allowed(&self) -> bool {
                #frisian_body
            }

            /// EK polling station (STEMBUREAU) for this province
            pub fn sb_district(&self) -> ElectoralDistrict {
                match self { #(#sb_arms)* }
            }

            pub fn ps_districts(&self) -> &'static [ElectoralDistrict] {
                match self { #(#ps_arms)* }
            }

            pub fn from_code(code: &str) -> Option<Self> {
                Self::ALL.iter().find(|x| x.code() == code).copied()
            }
        }
    };

    write_file(tokens, "provinces_generated.rs", out_dir);
}

fn write_water_councils_file(out_dir: &Path, districts: &Districts) {
    let variants: Vec<_> = districts
        .water_authority
        .iter()
        .map(|r| format_ident!("{}", to_ident(&r.name)))
        .collect();

    let code_arms = districts.water_authority.iter().map(|r| {
        let id = format_ident!("{}", to_ident(&r.name));
        let code = format!("ws{}", num(r));
        quote! { WaterCouncil::#id => #code, }
    });

    let title_arms = districts.water_authority.iter().map(|r| {
        let id = format_ident!("{}", to_ident(&r.name));
        let title = r.name.as_ref();
        quote! { WaterCouncil::#id => #title, }
    });

    let region_number_arms = districts.water_authority.iter().map(|r| {
        let id = format_ident!("{}", to_ident(&r.name));
        let number = num(r);
        quote! { WaterCouncil::#id => #number, }
    });

    let frisian_matches: Vec<_> = districts
        .water_authority
        .iter()
        .filter(|r| r.frysian_export_allowed)
        .map(|r| {
            let id = format_ident!("{}", to_ident(&r.name));
            quote! { WaterCouncil::#id }
        })
        .collect();
    let frisian_body = if frisian_matches.is_empty() {
        quote! { false }
    } else {
        quote! { matches!(self, #(#frisian_matches)|*) }
    };

    let ws_district_arms = districts.water_authority.iter().map(|r| {
        let ws_id = format_ident!("{}", to_ident(&r.name));
        let ws_num = num(r);
        let (wsk_r, _) = districts
            .water_authority_electoral_district
            .iter()
            .find(|(_, wn)| *wn == ws_num)
            .unwrap_or_else(|| panic!("No kieskring for waterschap {}", to_ident(&r.name)));
        let wsk_id = format_ident!("Ws{}", to_ident(&wsk_r.name));
        quote! { WaterCouncil::#ws_id => &[ElectoralDistrict::#wsk_id], }
    });

    let tokens = quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
        pub enum WaterCouncil {
            #(#variants,)*
        }

        impl WaterCouncil {
            pub const ALL: &[WaterCouncil] = &[#(WaterCouncil::#variants,)*];

            pub fn code(&self) -> &'static str {
                match self { #(#code_arms)* }
            }

            pub fn title(&self) -> &'static str {
                match self { #(#title_arms)* }
            }

            pub fn region_number(&self) -> u16 {
                match self { #(#region_number_arms)* }
            }

            pub fn frisian_export_allowed(&self) -> bool {
                #frisian_body
            }

            /// WATERSCHAP_KIESKRING districts per WaterCouncil (always one kieskring)
            pub fn ws_districts(&self) -> &'static [ElectoralDistrict] {
                match self { #(#ws_district_arms)* }
            }

            pub fn from_code(code: &str) -> Option<Self> {
                Self::ALL.iter().find(|x| x.code() == code).copied()
            }
        }
    };

    write_file(tokens, "water_councils_generated.rs", out_dir);
}

fn write_file(tokens: TokenStream, filename: &'static str, out_dir: &Path) {
    let file = syn::parse2(tokens).expect("Failed to parse generated tokens");
    let path = out_dir.join(filename);
    std::fs::write(&path, prettyplease::unparse(&file)).expect("Failed to write generated file");
}

#[cfg(test)]
mod tests {
    use eml_nl::documents::master_election_tree::MetCommittee;

    use super::*;

    #[test]
    fn districts_are_of_correct_category() {
        let districts = Districts::parse_from_file(Path::new("../../MasterElectionTree.xml"));
        assert!(
            districts
                .provinces_and_colleges
                .iter()
                .all(|d| d.key.category == RegionCategory::Province
                    || d.key.category == RegionCategory::ElectoralCollege)
        );
        assert!(
            districts
                .province_polling_stations
                .iter()
                .all(|(d, _)| d.key.category == RegionCategory::ProvincePollingStation)
        );
        assert!(
            districts
                .electoral_districts
                .iter()
                .all(|d| d.key.category == RegionCategory::ElectoralDistrict)
        );
        assert!(
            districts
                .province_electoral_district
                .iter()
                .all(|(d, _)| d.key.category == RegionCategory::ProvinceElectoralDistrict)
        );
        assert!(
            districts
                .municipality
                .iter()
                .all(|(d, _, _)| d.key.category == RegionCategory::Municipality)
        );
        assert!(
            districts
                .water_authority
                .iter()
                .all(|d| d.key.category == RegionCategory::WaterAuthority)
        );
        assert!(
            districts
                .water_authority_electoral_district
                .iter()
                .all(|(d, _)| d.key.category == RegionCategory::WaterAuthorityElectoralDistrict)
        );
        assert!(
            districts
                .water_authority_municipality
                .iter()
                .all(|(d, _, _, _)| d.key.category == RegionCategory::WaterAuthorityMunicipality)
        );
    }

    fn temp_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("eks-utils-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn test_districts() -> Districts {
        Districts {
            provinces_and_colleges: vec![
                MetRegion::new("Provincie-A", RegionCategory::Province)
                    .with_number(1)
                    .with_committees(vec![MetCommittee::new(
                        eml_nl::utils::ElectionCategory::EK,
                        eml_nl::utils::CommitteeCategory::CSB,
                    )]),
                MetRegion::new("Kiescollege-B", RegionCategory::ElectoralCollege).with_number(99),
            ],
            province_polling_stations: vec![
                (
                    MetRegion::new("Stembureau-A", RegionCategory::ProvincePollingStation),
                    1,
                ),
                (
                    MetRegion::new("Stembureau-B", RegionCategory::ProvincePollingStation)
                        .with_number(2),
                    99,
                ),
            ],
            electoral_districts: vec![MetRegion::new(
                "Kieskring-A",
                RegionCategory::ElectoralDistrict,
            )],
            province_electoral_district: vec![(
                MetRegion::new(
                    "Provinciaal-Kieskring-A",
                    RegionCategory::ProvinceElectoralDistrict,
                )
                .with_number(2),
                1,
            )],
            municipality: vec![(
                MetRegion::new("Gemeente-A", RegionCategory::Municipality).with_number(1),
                2,
                1,
            )],
            water_authority: vec![
                MetRegion::new("Waterschap-A", RegionCategory::WaterAuthority).with_number(3),
            ],
            water_authority_electoral_district: vec![(
                MetRegion::new(
                    "Waterschap-Kieskring-A",
                    RegionCategory::WaterAuthorityElectoralDistrict,
                ),
                3,
            )],
            water_authority_municipality: vec![
                (
                    MetRegion::new(
                        "Waterschap-Gemeente-A",
                        RegionCategory::WaterAuthorityMunicipality,
                    ),
                    "GmGemeenteA".to_string(),
                    3,
                    false,
                ),
                (
                    MetRegion::new(
                        "Waterschap-Gemeente-B",
                        RegionCategory::WaterAuthorityMunicipality,
                    ),
                    "GmGemeenteB".to_string(),
                    3,
                    true,
                ),
            ],
        }
    }

    #[test]
    fn all_entries_contains_every_district() {
        let districts = test_districts();
        let entries = districts.all_entries();
        let variants: std::collections::HashSet<&str> =
            entries.iter().map(|e| e.variant.as_str()).collect();

        assert!(variants.contains("ProvincieA"));
        assert!(variants.contains("KiescollegeB"));
        assert!(variants.contains("SbStembureauA"));
        assert!(variants.contains("SbStembureauB"));
        assert!(variants.contains("TkKieskringA"));
        assert!(variants.contains("PsProvinciaalKieskringA"));
        assert!(variants.contains("GmGemeenteA"));
        assert!(variants.contains("GmGemeenteBFr"));
        assert!(variants.contains("WsWaterschapKieskringA"));

        // The "GmGemeenteA" is both a water authority municipality and a normal
        // municipality, but it should not produce a duplicate entry
        assert_eq!(
            entries.len(),
            9,
            "all_entries should have one entry per district, without duplicates"
        );
    }

    #[test]
    fn generate_districts() {
        let districts = test_districts();
        let dir = temp_dir();
        write_districts_file(&dir, &districts);
        let districts_file = std::fs::read_to_string(dir.join("districts_generated.rs"))
            .expect("read generated districts");

        // Every district the struct produces an enum variant for should show up
        // in the generated file, by variant name, code and title
        for entry in districts.all_entries() {
            assert!(
                districts_file.contains(&entry.variant),
                "generated districts file is missing variant {}",
                entry.variant
            );
            assert!(
                districts_file.contains(&entry.code),
                "generated districts file is missing code {}",
                entry.code
            );
            assert!(
                districts_file.contains(entry.region.name.as_ref()),
                "generated districts file is missing title {}",
                entry.region.name
            );
        }

        // The CSB committee defined on Provincie-A should show up
        assert!(
            districts_file.contains("CommitteeCategory::CSB"),
            "generated districts file is missing the CSB committee"
        );
    }

    #[test]
    fn generate_provinces() {
        let districts = test_districts();
        let dir = temp_dir();
        write_provinces_file(&dir, &districts);
        let provinces_file = std::fs::read_to_string(dir.join("provinces_generated.rs"))
            .expect("read generated provinces");

        // Every PROVINCIE (KIESCOLLEGEs are excluded) should show up in the
        // generated file, by variant name, code and title
        for p in districts
            .provinces_and_colleges
            .iter()
            .filter(|r| r.key.category == RegionCategory::Province)
        {
            let variant = to_ident(&p.name);
            assert!(
                provinces_file.contains(&variant),
                "generated provinces file is missing variant {variant}"
            );
            assert!(
                provinces_file.contains(&format!("prov{}", num(p))),
                "generated provinces file is missing code for {}",
                p.name
            );
            assert!(
                provinces_file.contains(p.name.as_ref()),
                "generated provinces file is missing title {}",
                p.name
            );
        }

        // KIESCOLLEGEs should not show up in the generated file at all
        assert!(
            !provinces_file.contains("KiescollegeB"),
            "generated provinces file should not contain KIESCOLLEGE variant KiescollegeB"
        );
        assert!(
            !provinces_file.contains("kc99"),
            "generated provinces file should not contain KIESCOLLEGE code"
        );
    }

    #[test]
    fn generate_water_councils() {
        let districts = test_districts();
        let dir = temp_dir();
        write_water_councils_file(&dir, &districts);
        let water_councils_file = std::fs::read_to_string(dir.join("water_councils_generated.rs"))
            .expect("read generated water councils");

        // Every WATERSCHAP should show up in the generated file, by variant
        // name, code and title
        for r in &districts.water_authority {
            let variant = to_ident(&r.name);
            assert!(
                water_councils_file.contains(&variant),
                "generated water councils file is missing variant {variant}"
            );
            assert!(
                water_councils_file.contains(&format!("ws{}", num(r))),
                "generated water councils file is missing code for {}",
                r.name
            );
            assert!(
                water_councils_file.contains(r.name.as_ref()),
                "generated water councils file is missing title {}",
                r.name
            );
        }
    }
}
