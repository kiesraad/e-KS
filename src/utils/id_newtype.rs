//! Macro for defining UUID-backed ID newtypes with common trait impls.

/// Define a UUID-backed ID newtype with common trait implementations.
#[macro_export]
macro_rules! id_newtype {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident
    ) => {
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash,
            Ord, PartialOrd,
            serde::Serialize, serde::Deserialize,
        )]
        #[serde(transparent)]
        $vis struct $name(uuid::Uuid);

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl From<uuid::Uuid> for $name {
            fn from(v: uuid::Uuid) -> Self { Self(v) }
        }

        impl std::str::FromStr for $name {
            type Err = $crate::form::ValidationError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(uuid::Uuid::parse_str(s).map_err(|_| $crate::form::ValidationError::InvalidValue)?))
            }
        }

        impl From<$name> for uuid::Uuid {
            fn from(v: $name) -> Self { v.0 }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl $name {
            pub fn new() -> Self {
                Self(uuid::Uuid::new_v4())
            }

            pub fn uuid(&self) -> uuid::Uuid {
                self.0
            }
        }
    };
}
