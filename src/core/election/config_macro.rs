/// Macro to define election configs, used in configs.rs
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
                election_date: $election_date:expr,
                eligible_date_of_birth: $eligible_date_of_birth:expr,
                electoral_districts: $electoral_districts:expr $(,)?,
                nineteen_or_more_seats: $nineteen_or_more_seats:expr
            }
        ),* $(,)?
    ) => {
	    /// Active election configurations and ruleset for the application.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        pub enum ElectionConfig {
            $(
                $name $(($binding_ty))?,
            )*
        }

        impl ElectionConfig {
            /// Short code identifying the election type (without region), used in forms.
            pub fn code(&self) -> &'static str {
                #[allow(unused)]
                match self {
                    $(
                        Self::$name $(($binding))? => stringify!($name),
                    )*
                }
            }

            /// Returns the region code (province or water council code), if any.
            pub fn region_code(&self) -> Option<&'static str> {
                #[allow(unused)]
                match self {
                    $(
                        Self::$name $(($binding))? => {
                            #[allow(unused_mut, unused_assignments)]
                            let mut result: Option<&'static str> = None;
                            $( result = Some($binding.code()); )?
                            result
                        },
                    )*
                }
            }

            /// Returns the region title (province or water council name), if any.
            pub fn region_title(&self, locale: AnyLocale) -> Option<&'static str> {
                #[allow(unused)]
                match self {
                    $(
                        Self::$name $(($binding))? => {
                            #[allow(unused_mut, unused_assignments)]
                            let mut result: Option<&'static str> = None;
                            $( result = Some($binding.title(locale)); )?
                            result
                        },
                    )*
                }
            }

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

            pub fn election_date(&self) -> NaiveDate {
                #[allow(unused)]
                match self {
                    $(
                        Self::$name $(($binding))? => $election_date,
                    )*
                }
            }

            pub fn eligible_date_of_birth(&self) -> NaiveDate {
                #[allow(unused)]
                match self {
                    $(
                        Self::$name $(($binding))? => $eligible_date_of_birth,
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

            pub fn nineteen_or_more_seats(&self) -> bool {
                #[allow(unused)]
                match self {
                    $(
                        Self::$name $(($binding))? => $nineteen_or_more_seats,
                    )*
                }
            }

            /// Parse an election code plus optional region code into a variant.
            /// Variants without a region ignore the `region` argument; variants
            /// with one return `None` if `region` is missing or invalid.
            #[allow(unused_variables)]
            pub fn from_code_and_region(code: &str, region: Option<&str>) -> Option<Self> {
                $(
                    if code == stringify!($name) {
                        return Some(Self::$name $((<$binding_ty>::from_code(region?)?))?);
                    }
                )*
                None
            }

        }
    };
}

pub(crate) use define_elections;
