/// This build script generates Rust bindings for the `tc` API.
fn main() {
    let api_dir = "src";

    let bindings = bindgen::Builder::default()
        .header(format!("build.h"))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = std::path::PathBuf::from(api_dir);
    bindings
        .write_to_file(out_path.join("tc_bindings.rs"))
        .expect("Couldn't write bindings!");
}
