use std::{
    env,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use saphyr::{LoadableYamlNode, Scalar, Yaml};

fn flatten_yaml(prefix: &str, yaml: &Yaml, file: &mut BufWriter<File>) {
    match yaml {
        Yaml::Mapping(h) => {
            for (k, v) in h {
                if let Yaml::Value(Scalar::String(key)) = k {
                    if prefix.is_empty() {
                        flatten_yaml(key, v, file);
                    } else {
                        flatten_yaml(&format!("{}.{}", prefix, key), v, file);
                    }
                }
            }
        }
        Yaml::Value(Scalar::String(s)) => {
            writeln!(file, "    (\"{prefix}\") => {{ r###\"{s}\"### }};").unwrap();
        }
        _ => {
            println!("cargo:warning=Unsupported YAML value for key '{prefix}'");
        }
    }
}

fn load_locales() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("locales.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    for lang in &["En", "Nl"] {
        let locale_path = format!("./locales/{}.yml", lang.to_lowercase());
        println!("cargo:rerun-if-changed={locale_path}");

        if !Path::new(&locale_path).exists() {
            panic!("Locale file '{locale_path}' does not exist");
        }

        let yaml = std::fs::read_to_string(locale_path).expect("Failed to read locale file");
        let docs = match Yaml::load_from_str(&yaml) {
            Ok(d) => d,
            Err(e) => {
                panic!("Failed to load translations: {e}");
            }
        };

        writeln!(
            file,
            "#[macro_export]\nmacro_rules! inner_t_{} {{\n",
            lang.to_lowercase()
        )
        .unwrap();

        flatten_yaml("", &docs[0], &mut file);

        #[cfg(feature = "dev-features")]
        writeln!(
            file,
            "($other:literal) => {{
                concat!(\"[\", $other, \"]\")
            }};\n}}\npub use inner_t_{} as t_{};\n",
            lang.to_lowercase(),
            lang.to_lowercase()
        )
        .unwrap();

        #[cfg(not(feature = "dev-features"))]
        writeln!(
            file,
            "($other:literal) => {{
                ::core::compile_error!(concat!(\"unknown translation key: \", $other))
            }};\n}}\npub use inner_t_{} as t_{};\n",
            lang.to_lowercase(),
            lang.to_lowercase()
        )
        .unwrap();
    }
}

fn main() {
    std::fs::create_dir_all("./frontend/static")
        .expect("Failed to create frontend/static directory");

    #[cfg(feature = "memory-serve")]
    memory_serve::load_directory("./frontend/static");

    load_locales();
}
