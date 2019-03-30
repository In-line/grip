fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    /*
    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_include_guard("GRIP_FFI_H")
        .with_include("amxxmodule.h")
        .exclude_item("Cell")
        .rename_item("Cell", "cell")
        .with_language(cbindgen::Language::Cxx)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("../cpp/ffi.h");
        */
}
