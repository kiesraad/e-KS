use std::path::Path;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::write_file;
use crate::{
    districts::Districts,
    utils::{num, to_ident},
};

#[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
pub(crate) fn write_districts_file(out_dir: &Path, districts: &Districts) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::districts::test_support::{temp_dir, test_districts};

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
}
