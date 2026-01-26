include!("locales/utils/collect_locale_files.rs");
include!("locales/utils/naive_yaml_parse.rs");
include!("locales/utils/load_locales.rs");

fn main() {
    std::fs::create_dir_all("./frontend/static")
        .expect("Failed to create frontend/static directory");

    #[cfg(feature = "memory-serve")]
    memory_serve::load_directory("./frontend/static");

    load_locales(&std::env::var("OUT_DIR").unwrap());
}
