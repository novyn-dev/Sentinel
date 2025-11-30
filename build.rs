use std::path::PathBuf;

extern crate bindgen;
extern crate cc;

fn main() {
    cc::Build::new()
        .file("c_code/elf/predict.c")
        .file("c_code/exe/predict.c")
        .file("c_code/helper.c")
        .compile("predict");

    // rerun cuz i dont wanna call `cargo clean` everytime like a maniac
    println!("cargo:rerun-if-changed=c_code/elf/predict.c");

    // link rust to the library
    println!("cargo:rustc-link-search=native={}", std::env::var("OUT_DIR").unwrap());
    println!("cargo:rustc-link-lib=static=predict");

    // link xgboost
    println!("cargo:rustc-link-search=native=/usr/local/lib");
    println!("cargo:rustc-link-lib=dylib=xgboost");

    // link LIEF
    println!("cargo:rustc-link-lib=dylib=LIEF");

    // link stdc++
    println!("cargo:rustc-link-lib=dylib=stdc++");

    let bindings = bindgen::Builder::default()
        .header("./wrapper.h")
        .generate()
        .expect("Couldn't generate the bindings");

    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write the bindings");
}
