/// Generates `locales.rs` in `out_dir` with translation macros and PHF maps.
///
/// The output includes data for the hard-coded language list and emits
/// `cargo:rerun-if-changed` for each locale file.
pub fn load_locales(out_dir: &str) {
    use std::io::Write;
    
    let path = std::path::Path::new(out_dir).join("locales.rs");
    let mut file = std::io::BufWriter::new(std::fs::File::create(&path).unwrap());

    for lang in &["en", "nl"] {
        let mut map: phf_codegen::Map<String> = phf_codegen::Map::new();
        let locale_dir = std::path::Path::new("./locales").join(lang);
        let locale_files = collect_locale_files(&locale_dir);

        writeln!(
            file,
            "/// Translate a literal key to a raw localized string for `{lang}`.\n#[macro_export]\nmacro_rules! inner_t_{} {{\n",
            lang
        )
        .unwrap();

        for locale_path in locale_files {
            println!("cargo:rerun-if-changed={}", locale_path.display());
            let yaml = std::fs::read_to_string(&locale_path).expect("Failed to read locale file");
            let key = locale_path.file_stem().unwrap().to_str().unwrap();
            let entries = naive_yaml_parse(key, &yaml);

            for (key, mut value) in entries {
                value = format!("r###\"{value}\"###");
                writeln!(file, "    (\"{key}\") => {{ {value} }};").unwrap();
                map.entry(key, value);
            }
        }

        writeln!(
            file,
            "($other:literal) => {{
                concat!(\"[\", $other, \"]\")
            }};\n}}\npub use inner_t_{} as t_{};\n",
            lang, lang
        )
        .unwrap();

        writeln!(
            &mut file,
            "pub static LOCALE_{}: phf::Map<&'static str, &'static str> = {};",
            lang.to_uppercase(),
            map.build()
        )
        .unwrap();
    }
}
