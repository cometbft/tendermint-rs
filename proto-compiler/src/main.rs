use std::env::var;
use std::path::PathBuf;
use tempdir::TempDir;

mod functions;
use functions::{copy_files, find_proto_files, generate_tendermint_lib, get_commitish};

mod constants;
use constants::{
    CUSTOM_FIELD_ATTRIBUTES, CUSTOM_TYPE_ATTRIBUTES, TENDERMINT_COMMITISH, TENDERMINT_REPO,
};

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tendermint_lib_target = root.join("../proto/src/tendermint.rs");
    let target_dir = root.join("../proto/src/prost");
    let out_dir = var("OUT_DIR")
        .map(PathBuf::from)
        .or_else(|_| TempDir::new("tendermint_proto_out").map(|d| d.into_path()))
        .unwrap();
    let tendermint_dir = var("TENDERMINT_DIR").unwrap_or_else(|_| "target/tendermint".to_string());

    println!(
        "[info] => Fetching {} at {} into {}",
        TENDERMINT_REPO, TENDERMINT_COMMITISH, tendermint_dir
    );
    get_commitish(
        &PathBuf::from(&tendermint_dir),
        TENDERMINT_REPO,
        TENDERMINT_COMMITISH,
    ); // This panics if it fails.

    let proto_paths = [format!("{}/proto", tendermint_dir)];
    let proto_includes_paths = [
        format!("{}/proto", tendermint_dir),
        format!("{}/third_party/proto", tendermint_dir),
    ];
    // List available proto files
    let protos = find_proto_files(proto_paths.to_vec());
    // List available paths for dependencies
    let includes: Vec<PathBuf> = proto_includes_paths.iter().map(PathBuf::from).collect();

    // Compile proto files with added annotations, exchange prost_types to our own
    let mut pb = prost_build::Config::new();
    pb.out_dir(&out_dir);
    for type_attribute in CUSTOM_TYPE_ATTRIBUTES {
        pb.type_attribute(type_attribute.0, type_attribute.1);
    }
    for field_attribute in CUSTOM_FIELD_ATTRIBUTES {
        pb.field_attribute(field_attribute.0, field_attribute.1);
    }
    pb.compile_well_known_types();
    // The below in-place path redirection removes the Duration and Timestamp structs from
    // google.protobuf.rs. We replace them with our own versions that have valid doctest comments.
    // See also https://github.com/danburkert/prost/issues/374 .
    pb.extern_path(
        ".google.protobuf.Duration",
        "super::super::google::protobuf::Duration",
    );
    pb.extern_path(
        ".google.protobuf.Timestamp",
        "super::super::google::protobuf::Timestamp",
    );
    println!("[info] => Creating structs.");
    pb.compile_protos(&protos, &includes).unwrap();

    println!("[info] => Removing old structs and copying new structs.");
    copy_files(&out_dir, &target_dir); // This panics if it fails.
    generate_tendermint_lib(&out_dir, &tendermint_lib_target);

    println!("[info] => Done!");
}
