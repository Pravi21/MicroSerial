use std::env;
use std::path::PathBuf;

fn main() {
    let core_dir = PathBuf::from("../core");
    println!(
        "cargo:rerun-if-changed={}",
        core_dir.join("include/MicroSerial/ms_core.h").display()
    );
    println!("cargo:rerun-if-changed={}", core_dir.join("src").display());
    println!(
        "cargo:rerun-if-changed={}",
        core_dir.join("CMakeLists.txt").display()
    );

    let dst = cmake::Config::new(&core_dir)
        .define("BUILD_TESTING", "OFF")
        .profile("Release")
        .build();

    let lib_dir = if dst.join("lib64").exists() {
        dst.join("lib64")
    } else {
        dst.join("lib")
    };
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=microserial_core");
    println!("cargo:rustc-link-lib=dylib=pthread");

    let header_path = core_dir.join("include/MicroSerial/ms_core.h");
    let bindings = bindgen::Builder::default()
        .header(header_path.to_string_lossy().into_owned())
        .allowlist_function("ms_.*")
        .allowlist_type("ms_.*")
        .allowlist_var("MS_.*")
        .generate()
        .expect("unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("could not write bindings");
}
