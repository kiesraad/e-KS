fn main() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR environment variable not set");
    let out_dir = std::path::Path::new(&out_dir);

    std::fs::create_dir_all("./frontend/static")
        .expect("Failed to create frontend/static directory");

    #[cfg(feature = "memory-serve")]
    memory_serve::load_directory("./frontend/static");

    eks_locales::load_locales(out_dir, std::path::Path::new("./locales"));
    eks_districts_codegen::generate_election_tree(out_dir);
}
