include!("tooling/typst/generate_typst_files.rs");
include!("tooling/locales/collect_locale_files.rs");
include!("tooling/locales/naive_yaml_parse.rs");
include!("tooling/locales/load_locales.rs");

fn main() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR environment variable not set");

    std::fs::create_dir_all("./frontend/static")
        .expect("Failed to create frontend/static directory");

    #[cfg(feature = "memory-serve")]
    memory_serve::load_directory("./frontend/static");

    // Re-run build script if the embed-typst feature is toggled, since it controls whether the embedded Typst assets are included in the build.
    println!("cargo::rerun-if-env-changed=CARGO_FEATURE_EMBED_TYPST");

    #[cfg(feature = "embed-typst")]
    generate_typst_files_during_build(&out_dir).expect("failed to prepare embedded Typst assets");

    load_locales(&out_dir);
}
