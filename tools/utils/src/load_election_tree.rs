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
struct Districts<'a> {
    /// EK/PS: PROVINCIE + KIESCOLLEGE
    ek: Vec<&'a MetRegion>,
    /// EK: PROVINCIAAL_STEMBUREAU; tuple is (region, parent EK number)
    sb: Vec<(&'a MetRegion, u16)>,
    /// TK: KIESKRING
    tk: Vec<&'a MetRegion>,
    /// PS: PROVINCIAAL_KIESKRING; tuple is (region, grandparent PROVINCIE number)
    ps: Vec<(&'a MetRegion, u16)>,
    /// TK/PS: GEMEENTE; tuple is (region, parent PROVINCIAAL_KIESKRING number, grandparent PROVINCIE number)
    gm: Vec<(&'a MetRegion, u16, u16)>,
    /// WS: WATERSCHAP
    ws: Vec<&'a MetRegion>,
    /// WS: WATERSCHAP_KIESKRING; tuple is (region, parent WATERSCHAP number)
    wsk: Vec<(&'a MetRegion, u16)>,
    /// WS: WATERSCHAP_GEMEENTE references; tuple is (WATERSCHAP_GEMEENTE region, base Gm ident, parent WATERSCHAP number, FrysianExportAllowed)
    /// When FrysianExportAllowed differs from the PS gemeente, a separate Fr-suffixed variant is generated
    ws_gm: Vec<(&'a MetRegion, String, u16, bool)>,
}

/// One ElectoralDistrict variant with its precomputed names
struct RegionEntry<'a> {
    /// Rust enum variant name, e.g. "SbGroningen", "GmLeeuwarden"
    variant: String,
    /// Stable typed code for use in e.g. the EML API, e.g. "sb2", "gm358"
    code: String,
    region: &'a MetRegion,
}

impl<'a> Districts<'a> {
    /// All districts in enum declaration order, with precomputed variant name and code
    fn all_entries(&'a self) -> Vec<RegionEntry<'a>> {
        let mut entries = Vec::new();
        for r in &self.ek {
            let code_prefix = if r.key.category == RegionCategory::Province {
                "prov"
            } else {
                "kc"
            };
            entries.push(RegionEntry {
                variant: to_ident(&r.name),
                code: format!("{}{}", code_prefix, num(r)),
                region: r,
            });
        }
        for (r, _) in &self.sb {
            entries.push(RegionEntry {
                variant: format!("Sb{}", to_ident(&r.name)),
                code: format!("sb{}", num(r)),
                region: r,
            });
        }
        for r in &self.tk {
            entries.push(RegionEntry {
                variant: format!("Tk{}", to_ident(&r.name)),
                code: format!("tk{}", num(r)),
                region: r,
            });
        }
        for (r, _) in &self.ps {
            entries.push(RegionEntry {
                variant: format!("Ps{}", to_ident(&r.name)),
                code: format!("pk{}", num(r)),
                region: r,
            });
        }
        // The "Fr" suffix indicates a municipality has FrysianExportAllowed true.
        // Some municipalities appear both in Fryslan and in another water council,
        // so we use the suffix to remember when Frisian export is allowed.
        for (r, _, _) in &self.gm {
            let suffix = if r.frysian_export_allowed { "Fr" } else { "" };
            entries.push(RegionEntry {
                variant: format!("Gm{}{}", to_ident(&r.name), suffix),
                code: format!("gm{}{}", num(r), suffix.to_lowercase()),
                region: r,
            });
        }
        for (r, ws_num) in &self.wsk {
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
        for (gm_region, gm_ident, _, frisian_flag) in &self.ws_gm {
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

impl<'a> From<&'a MasterElectionTree> for Districts<'a> {
    fn from(tree: &'a MasterElectionTree) -> Self {
        let staat = &tree.root;
        let mut d = Districts::default();

        // Filter helper: iterate child regions of a given category
        let of = |cat: RegionCategory| move |r: &&MetRegion| r.key.category == cat;

        // PROVINCIE: EK provinces with their nested structure
        for prov in staat.subregions.iter().filter(of(RegionCategory::Province)) {
            let prov_num = num(prov);
            d.ek.push(prov);
            for sb in prov
                .subregions
                .iter()
                .filter(of(RegionCategory::ProvincePollingStation))
            {
                d.sb.push((sb, prov_num));
                for ks in sb
                    .subregions
                    .iter()
                    .filter(of(RegionCategory::ElectoralDistrict))
                {
                    d.tk.push(ks);
                    for pk in ks
                        .subregions
                        .iter()
                        .filter(of(RegionCategory::ProvinceElectoralDistrict))
                    {
                        let pk_num = num(pk);
                        d.ps.push((pk, prov_num));
                        for gm in pk
                            .subregions
                            .iter()
                            .filter(of(RegionCategory::Municipality))
                            .filter(|r| num(r) != NBSB_REGION_NUMBER)
                        {
                            d.gm.push((gm, pk_num, prov_num));
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
            d.ek.push(kc);
            for sb in kc
                .subregions
                .iter()
                .filter(of(RegionCategory::ProvincePollingStation))
            {
                d.sb.push((sb, kc_num));
            }
        }

        // KIESKRING directly under STAAT (Bonaire special case)
        for ks in staat
            .subregions
            .iter()
            .filter(of(RegionCategory::ElectoralDistrict))
        {
            d.tk.push(ks);
        }

        // WATERSCHAP: WS water authorities with their municipalities
        // Build a CBS-code -> Gm ident lookup from already-collected PS municipalities
        let cbs_to_gm_ident: std::collections::HashMap<u16, String> =
            d.gm.iter()
                .map(|(g, _, _)| (num(g), format!("Gm{}", to_ident(&g.name))))
                .collect();
        for ws in staat
            .subregions
            .iter()
            .filter(of(RegionCategory::WaterAuthority))
        {
            let ws_num = num(ws);
            d.ws.push(ws);
            for wsk in ws
                .subregions
                .iter()
                .filter(of(RegionCategory::WaterAuthorityElectoralDistrict))
            {
                d.wsk.push((wsk, ws_num));
                for gm in wsk
                    .subregions
                    .iter()
                    .filter(of(RegionCategory::WaterAuthorityMunicipality))
                {
                    if let Some(gm_ident) = cbs_to_gm_ident.get(&num(gm)) {
                        d.ws_gm
                            .push((gm, gm_ident.clone(), ws_num, gm.frysian_export_allowed));
                    }
                }
            }
        }

        // Sort every collection except for municipalities by its RegionNumber,
        // so ordering doesn't depend on the document order of MasterElectionTree.xml
        d.ek.sort_by_key(|r| num(r));
        d.sb.sort_by_key(|(r, _)| num(r));
        d.tk.sort_by_key(|r| num(r));
        d.ps.sort_by_key(|(r, _)| num(r));
        d.ws.sort_by_key(|r| num(r));
        d.wsk.sort_by_key(|(r, _)| num(r));

        d
    }
}

pub fn generate_election_tree(out_dir: &Path) {
    use eml_nl::io::EMLParsingMode;

    println!("cargo:rerun-if-changed=MasterElectionTree.xml");

    let xml = std::fs::read_to_string("MasterElectionTree.xml")
        .expect("MasterElectionTree.xml not found in workspace root");

    let tree = MasterElectionTree::parse_eml(&xml, EMLParsingMode::Strict)
        .expect("Failed to parse MasterElectionTree.xml");

    let districts = Districts::from(&tree);

    write_districts_file(out_dir, &districts);
    write_provinces_file(out_dir, &districts);
    write_water_councils_file(out_dir, &districts);
}

fn write_districts_file(out_dir: &Path, districts: &Districts<'_>) {
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
        .ek
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
    for r in &districts.ek {
        let r_id = format_ident!("{}", to_ident(&r.name));
        let (sb_r, _) = districts
            .sb
            .iter()
            .find(|(_, pn)| *pn == num(r))
            .unwrap_or_else(|| panic!("No polling station for EK district {}", to_ident(&r.name)));
        let sb_id = format_ident!("Sb{}", to_ident(&sb_r.name));
        sub_arms.push(quote! { Self::#r_id => &[ElectoralDistrict::#sb_id], });
    }
    for (ps_r, ps_prov_num) in &districts.ps {
        let ps_id = format_ident!("Ps{}", to_ident(&ps_r.name));
        let ps_pk_num = num(ps_r);
        let gm_ids: Vec<_> = districts
            .gm
            .iter()
            .filter(|(_, pk, prov)| *pk == ps_pk_num && *prov == *ps_prov_num)
            .map(|(g, _, _)| {
                let suffix = if g.frysian_export_allowed { "Fr" } else { "" };
                format_ident!("Gm{}{}", to_ident(&g.name), suffix)
            })
            .collect();
        sub_arms.push(quote! { Self::#ps_id => &[#(ElectoralDistrict::#gm_ids),*], });
    }
    for (wsk_r, ws_num) in &districts.wsk {
        let wsk_id = format_ident!("Ws{}", to_ident(&wsk_r.name));
        let gm_ids: Vec<_> = districts
            .ws_gm
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

fn write_provinces_file(out_dir: &Path, districts: &Districts<'_>) {
    // filter out KIESCOLLEGEs and keep only PROVINCIEs
    let raw_provinces: Vec<&MetRegion> = districts
        .ek
        .iter()
        .copied()
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
            .sb
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
            .ps
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

fn write_water_councils_file(out_dir: &Path, districts: &Districts<'_>) {
    let variants: Vec<_> = districts
        .ws
        .iter()
        .map(|r| format_ident!("{}", to_ident(&r.name)))
        .collect();

    let code_arms = districts.ws.iter().map(|r| {
        let id = format_ident!("{}", to_ident(&r.name));
        let code = format!("ws{}", num(r));
        quote! { WaterCouncil::#id => #code, }
    });

    let title_arms = districts.ws.iter().map(|r| {
        let id = format_ident!("{}", to_ident(&r.name));
        let title = r.name.as_ref();
        quote! { WaterCouncil::#id => #title, }
    });

    let region_number_arms = districts.ws.iter().map(|r| {
        let id = format_ident!("{}", to_ident(&r.name));
        let number = num(r);
        quote! { WaterCouncil::#id => #number, }
    });

    let frisian_matches: Vec<_> = districts
        .ws
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

    let ws_district_arms = districts.ws.iter().map(|r| {
        let ws_id = format_ident!("{}", to_ident(&r.name));
        let ws_num = num(r);
        let (wsk_r, _) = districts
            .wsk
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
