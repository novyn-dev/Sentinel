use std::path::PathBuf;

extern crate bindgen;
extern crate cc;

fn main() {
    // Get LIEF paths from environment or use defaults
    let lief_lib = std::env::var("LIEF_LIB_PATH")
        .unwrap_or_else(|_| "/usr/lib/".to_string());
    let lief_include = std::env::var("LIEF_INCLUDE_PATH")
        .unwrap_or_else(|_| "/usr/include/".to_string());
    let lief_wrapper_path = std::env::var("LIEF_WRAPPER_PATH")
        .unwrap_or_else(|_| "c_code/exe/".to_string());

    // same for xgboost
    let xgboost_lib = std::env::var("XGBOOST_LIB_PATH")
        .unwrap_or_else(|_| "/usr/lib/".to_string());
    let xgboost_include = std::env::var("XGBOOST_INCLUDE_PATH")
        .unwrap_or_else(|_| "/usr/include/".to_string());

    cc::Build::new()
        .file("c_code/elf/predict.c")
        .file("c_code/exe/predict.c")
        .file("c_code/helper.c")
        .include(&lief_include)
        .include(&xgboost_include)
        .compile("predict");

    // rerun cuz i dont wanna call `cargo clean` everytime like a maniac
    println!("cargo:rerun-if-changed=c_code");

    // link rust to the library
    println!("cargo:rustc-link-search=native={}", std::env::var("OUT_DIR").unwrap());
    println!("cargo:rustc-link-lib=static=predict");

    // link xgboost
    println!("cargo:rustc-link-search=native={xgboost_lib}");
    println!("cargo:rustc-link-lib=dylib=xgboost");

    // link LIEF
    println!("cargo:rustc-link-search=native={lief_lib}");
    println!("cargo:rustc-link-lib=dylib=LIEF");

    // link stdc++
    println!("cargo:rustc-link-lib=dylib=stdc++");

    // link lief_wrapper (my custom wrapper)
    println!("cargo:rustc-link-search=native={lief_wrapper_path}");
    println!("cargo:rustc-link-lib=dylib=lief_wrapper");

    let bindings = bindgen::Builder::default()
        .header("./wrapper.h")
        .generate()
        .expect("Couldn't generate the bindings");

    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write the bindings");
}
