/* Copyright (c) Fortanix, Inc.
 *
 * Licensed under the GNU General Public License, version 2 <LICENSE-GPL or
 * https://www.gnu.org/licenses/gpl-2.0.html> or the Apache License, Version
 * 2.0 <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0>, at your
 * option. This file may not be copied, modified, or distributed except
 * according to those terms. */

use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use rustc_version::Channel;
use std::env;

<<<<<<< HEAD
/// Retrieves or generates a metadata value used for symbol name mangling to ensure unique C symbols.
/// When building with Cargo, the metadata value is extracted from the OUT_DIR environment variable.
/// For Bazel builds, this method generate the suffix by hashing part of the crate OUT_DIR,
/// which are sufficient for ensuring symbol uniqueness.
fn get_compilation_symbol_suffix() -> String {
    let out_dir: std::path::PathBuf = std::env::var_os("OUT_DIR").unwrap().into();
    let mut out_dir_it_rev = out_dir.iter().rev();
    if out_dir_it_rev.next().map_or(false, |p| p == "out") {
        // If Cargo is used as build system.
        let crate_ = out_dir_it_rev
            .next()
            .expect("Expected OUT_DIR to have at least 2 components")
            .to_str()
            .expect("Expected second to last component of OUT_DIR to be a valid UTF-8 string");
        assert!(
            crate_.starts_with("mbedtls-"),
            "Expected second to last component of OUT_DIR to start with 'mbedtls-'"
        );
        return crate_[8..].to_owned(); // Return the part after "mbedtls-"
    } else if out_dir.iter().rfind(|p| *p == "bazel-out").is_some() {
        // If Bazel is used as build system.
        let mut hasher = DefaultHasher::new();
        // Reverse the iterator and hash until we find "bazel-out"
        for p in out_dir.iter().rev().take_while(|p| *p != "bazel-out") {
            p.hash(&mut hasher);
        }
        return format!("{:016x}", hasher.finish());
=======
/// Return the crate hash that Cargo will be passing to `rustc -C metadata=`.
// If there's a panic in this code block, that means Cargo's way of running the
// build script has changed, and this code should be updated to handle the new
// case.
fn get_compilation_metadata_hash() -> String {
    let out_dir: std::path::PathBuf = std::env::var_os("OUT_DIR").unwrap().into();
    let mut out_dir_it = out_dir.iter().rev();
    let next = out_dir_it.next().unwrap();

    if next == "out" {
        // Do the same thing as before
        let crate_ = out_dir_it.next().unwrap().to_string_lossy();
        assert!(crate_.starts_with("mbedtls-"));
        return crate_[8..].to_owned();
    } else if next == "_bs.out_dir" {
        let compiler_version = rustc_version::version().expect("Failed to get rustc version").to_string();
        let version = env!("CARGO_PKG_VERSION");
        let versioned_string = format!("mbedtls_{}", version);
        let metadata = vec!["".to_string()];
        let stable_crate_id = crate_id::StableCrateId::new(&versioned_string, false, metadata, compiler_version);

        return stable_crate_id.to_string();
>>>>>>> bazel compatible
    } else {
        panic!("unexpected OUT_DIR format: {}", out_dir.display());
    }
}


fn main() {
    // used for configuring rustdoc attrs for now
    if rustc_version::version_meta().is_ok_and(|v| v.channel == Channel::Nightly) {
        println!("cargo:rustc-cfg=nightly");
    }
    let symbol_suffix = get_compilation_symbol_suffix();
    println!("cargo:rustc-env=RUST_MBEDTLS_SYMBOL_SUFFIX={}", symbol_suffix);
    println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");


    let env_components = env::var("DEP_MBEDTLS_PLATFORM_COMPONENTS").unwrap();
    let mut sys_platform_components = HashMap::<_, HashSet<_>>::new();
    for mut kv in env_components.split(",").map(|component| component.splitn(2, "=")) {
        let k = kv.next().unwrap();
        let v = kv.next().unwrap();
        sys_platform_components.entry(k).or_insert_with(Default::default).insert(v);
        println!(r#"cargo:rustc-cfg=sys_{}="{}""#, k, v);
    }

    let mut b = cc::Build::new();
    let mbedtls_include = env::var_os("DEP_MBEDTLS_INCLUDE_BZL")
        .or(env::var_os("DEP_MBEDTLS_INCLUDE"))
        .unwrap();
    println!("Adding to include path: {}", mbedtls_include.to_str().unwrap());
    b.include(mbedtls_include);
    let config_file = env::var("DEP_MBEDTLS_CONFIG_H_BZL")
        .or(env::var("DEP_MBEDTLS_CONFIG_H"))
        .unwrap();
    println!("Defining MBEDTLS_CONFIG_FILE to be \"{}\"", config_file);
    b.define("MBEDTLS_CONFIG_FILE",
             Some(format!(r#""{}""#, config_file).as_str()));
    b.define("RUST_MBEDTLS_METADATA_HASH", Some(metadata_hash.as_str()));

    b.file("src/mbedtls_malloc.c");
    if sys_platform_components
        .get("c_compiler")
        .map_or(false, |comps| comps.contains("freestanding"))
    {
        b.flag("-U_FORTIFY_SOURCE")
            .define("_FORTIFY_SOURCE", Some("0"))
            .flag("-ffreestanding");
    }
    b.compile("librust-mbedtls.a");
}
