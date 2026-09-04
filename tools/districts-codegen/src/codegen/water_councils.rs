use std::path::Path;

use quote::{format_ident, quote};

use super::write_file;
use crate::{
    districts::Districts,
    utils::{num, to_ident},
};

#[allow(clippy::too_many_lines)]
pub(crate) fn write_water_councils_file(out_dir: &Path, districts: &Districts) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::districts::test_support::{temp_dir, test_districts};

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
