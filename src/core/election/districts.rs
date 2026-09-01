include!(concat!(env!("OUT_DIR"), "/districts_generated.rs"));

impl ElectoralDistrict {
    /// Returns the serde variant name, used as form value so that
    /// `serde_urlencoded` can deserialize it back into `ElectoralDistrict`.
    pub fn serde_name(&self) -> String {
        serde_json::to_value(self)
            .and_then(serde_json::from_value)
            .expect("unit enum variant serializes to a string")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ek_district_title_and_code_match() {
        assert_eq!(ElectoralDistrict::Utrecht.code(), "prov7");
        assert_eq!(ElectoralDistrict::Utrecht.region_number(), 7);
        assert_eq!(ElectoralDistrict::Utrecht.title(), "Utrecht");

        assert_eq!(ElectoralDistrict::Fryslan.code(), "prov2");
        assert_eq!(ElectoralDistrict::Fryslan.region_number(), 2);
        assert_eq!(ElectoralDistrict::Fryslan.title(), "Fryslân");

        assert_eq!(ElectoralDistrict::Buitenland.code(), "kc16");
        assert_eq!(ElectoralDistrict::Buitenland.region_number(), 16);
        assert_eq!(ElectoralDistrict::Buitenland.title(), "Buitenland");
    }

    #[test]
    fn ps_district_title_and_code_match() {
        assert_eq!(ElectoralDistrict::PsArnhem.code(), "pk1");
        assert_eq!(ElectoralDistrict::PsArnhem.region_number(), 1);
        assert_eq!(ElectoralDistrict::PsArnhem.title(), "Arnhem");

        assert_eq!(ElectoralDistrict::PsNijmegen.code(), "pk2");
        assert_eq!(ElectoralDistrict::PsNijmegen.region_number(), 2);
        assert_eq!(ElectoralDistrict::PsNijmegen.title(), "Nijmegen");
    }

    #[test]
    fn ws_district_title_and_code_match() {
        assert_eq!(ElectoralDistrict::WsHunzeEnAas.code(), "ws3");
        assert_eq!(ElectoralDistrict::WsHunzeEnAas.region_number(), 1);
        assert_eq!(ElectoralDistrict::WsHunzeEnAas.title(), "Hunze en Aa's");

        assert_eq!(ElectoralDistrict::WsAmstelGooiEnVecht.code(), "ws10");
        assert_eq!(ElectoralDistrict::WsAmstelGooiEnVecht.region_number(), 1);
        assert_eq!(
            ElectoralDistrict::WsAmstelGooiEnVecht.title(),
            "Amstel, Gooi en Vecht"
        );

        assert_eq!(ElectoralDistrict::WsFryslan.title(), "Fryslân");

        assert_eq!(
            ElectoralDistrict::WsDrentsOverijsselseDelta.region_number(),
            1
        );
        assert_eq!(ElectoralDistrict::WsLimburg.region_number(), 1);
    }

    #[test]
    fn electoral_districts_include_expected_code() {
        let districts = ElectoralDistrict::ek_districts();
        assert!(districts.contains(&ElectoralDistrict::Utrecht));
        assert_eq!(districts.len(), 16);
    }

    #[test]
    fn ps_zuid_holland_excludes_nbsb() {
        // Zuid-Holland's 's-Gravenhage PROVINCIAAL_KIESKRING has both the
        // 's-Gravenhage municipality and the NBSB pseudo-municipality, but
        // NBSB must not surface as a PS sub-district.
        assert!(
            crate::Province::ZuidHolland
                .ps_districts()
                .contains(&ElectoralDistrict::PsSGravenhage)
        );

        let sub_districts = ElectoralDistrict::PsSGravenhage.sub_districts();
        assert_eq!(sub_districts, &[ElectoralDistrict::GmSGravenhage]);
        assert!(!sub_districts.iter().any(|d| d.title() == "NBSB"));
    }

    #[test]
    fn similar_districts_have_same_title_but_differ_in_code() {
        // Fryslân appears as both a province (EK) and a waterschap; titles match,
        // codes differ by prefix: "prov2" vs "ws2".
        assert_eq!(
            ElectoralDistrict::Fryslan.title(),
            ElectoralDistrict::WsFryslan.title()
        );
        assert_ne!(
            ElectoralDistrict::Fryslan.code(),
            ElectoralDistrict::WsFryslan.code()
        );

        // Utrecht: EK district "prov7" vs PS district "pk{n}"
        assert_eq!(
            ElectoralDistrict::Utrecht.title(),
            ElectoralDistrict::PsUtrecht.title()
        );
        assert_ne!(
            ElectoralDistrict::Utrecht.code(),
            ElectoralDistrict::PsUtrecht.code()
        );

        // Limburg: province "prov12" vs waterschap "ws25"
        assert_eq!(
            ElectoralDistrict::Limburg.title(),
            ElectoralDistrict::WsLimburg.title()
        );
        assert_ne!(
            ElectoralDistrict::Limburg.code(),
            ElectoralDistrict::WsLimburg.code()
        );
    }
}
