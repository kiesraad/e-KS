include!("tooling/locales/collect_locale_files.rs");
include!("tooling/locales/naive_yaml_parse.rs");
include!("tooling/locales/load_locales.rs");

fn main() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR environment variable not set");

    std::fs::create_dir_all("./frontend/static")
        .expect("Failed to create frontend/static directory");

    #[cfg(feature = "memory-serve")]
    memory_serve::load_directory("./frontend/static");

    load_locales(&out_dir);
}
