#[macro_export]
macro_rules! impl_from_field {
    ($src:ty => $field:ident : $dst:ty) => {
        impl From<&$src> for $dst {
            fn from(src: &$src) -> Self {
                src.$field
            }
        }
    };
}
