//! Generates the districts and regions enums from `MasterElectionTree.xml`,
//! for use by the `eks` build script.
//!
//! - [`districts`] parses `MasterElectionTree.xml` into the [`districts::Districts`] model.
//! - [`codegen`] renders that model into the three `*_generated.rs` files.
//! - [`ident`] holds the small helpers both of the above rely on.
mod codegen;
mod districts;
mod utils;

use std::path::Path;

use districts::Districts;

pub fn generate_election_tree(out_dir: &Path) {
    println!("cargo:rerun-if-changed=MasterElectionTree.xml");
    let districts = Districts::parse_from_file(Path::new("MasterElectionTree.xml"));

    codegen::write_districts_file(out_dir, &districts);
    codegen::write_provinces_file(out_dir, &districts);
    codegen::write_water_councils_file(out_dir, &districts);
}
