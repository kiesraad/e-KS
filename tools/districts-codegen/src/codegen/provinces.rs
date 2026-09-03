use std::path::Path;

use eml_nl::{documents::master_election_tree::MetRegion, utils::RegionCategory};
use quote::{format_ident, quote};

use super::write_file;
use crate::{
    districts::Districts,
    utils::{num, to_ident},
};

#[allow(clippy::too_many_lines)]
pub(crate) fn write_provinces_file(out_dir: &Path, districts: &Districts) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::districts::test_support::{temp_dir, test_districts};

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
}
