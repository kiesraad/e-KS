#[macro_export]
macro_rules! id_newtype {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident
    ) => {
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(
            Debug, Default, Clone, Copy, PartialEq, Eq, Hash,
            serde::Serialize, serde::Deserialize,
            sqlx::Type,
        )]
        #[serde(transparent)]
        #[sqlx(transparent)]
        $vis struct $name(uuid::Uuid);

        impl From<uuid::Uuid> for $name {
            fn from(v: uuid::Uuid) -> Self { Self(v) }
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
