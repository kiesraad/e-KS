mod configs;
mod districts;
mod types;

pub use configs::ElectionConfig;
pub use districts::{ElectoralDistrict, Province};
pub use types::ElectionType;

macro_rules! define_elections {
    (
        $(
            $name:ident $( ( $binding:ident : $binding_ty:ty ) )? {
                election_type: $election_type:expr,
                titles: {
                    nl: $title_nl:expr,
                    fry: $title_fry:expr,
                    en: $title_en:expr $(,)?
                },
                nomination_day_date: $nomination_day_date:expr,
                electoral_districts: $electoral_districts:expr $(,)?
            }
        ),* $(,)?
    ) => {
	    /// Active election configurations and ruleset for the application.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum ElectionConfig {
            $(
                $name $(($binding_ty))?,
            )*
        }

        impl ElectionConfig {
            pub fn election_type(&self) -> ElectionType {
                #[allow(unused)]
                match self {
                    $(
                        Self::$name $(($binding))? => $election_type,
                    )*
                }
            }

            pub fn title(&self, locale: AnyLocale) -> &'static str {
                #[allow(unused)]
                match (self, locale) {
                    $(
                        (Self::$name $(($binding))?, AnyLocale::Nl) => $title_nl,
                        (Self::$name $(($binding))?, AnyLocale::Fry) => $title_fry,
                        (Self::$name $(($binding))?, AnyLocale::En) => $title_en,
                    )*
                }
            }

            pub fn nomination_day_date(&self) -> NaiveDate {
                #[allow(unused)]
                match self {
                    $(
                        Self::$name $(($binding))? => $nomination_day_date,
                    )*
                }
            }

            pub fn electoral_districts(&self) -> &'static [ElectoralDistrict] {
                match self {
                    $(
                        Self::$name $(($binding))? => $electoral_districts,
                    )*
                }
            }
        }
    };
}

use define_elections;
