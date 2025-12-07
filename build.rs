use std::path::PathBuf;
extern crate bindgen;
extern crate cc;

fn main() {
    let lief_lib = std::env::var("LIEF_LIB_PATH")
        .unwrap_or_else(|_| "/usr/lib/".to_string());
    let lief_include = std::env::var("LIEF_INCLUDE_PATH")
        .unwrap_or_else(|_| "/usr/include/".to_string());
    let lief_wrapper_path = std::env::var("LIEF_WRAPPER_PATH")
        .unwrap_or_else(|_| "c_code/exe/".to_string());
    
    let xgboost_lib = std::env::var("XGBOOST_LIB_PATH")
        .unwrap_or_else(|_| "/usr/lib/".to_string());
    let xgboost_include = std::env::var("XGBOOST_INCLUDE_PATH")
        .unwrap_or_else(|_| "/usr/include/".to_string());
    
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    
    let elf_predict = PathBuf::from(&manifest_dir).join("c_code/elf/predict.c");
    let exe_predict = PathBuf::from(&manifest_dir).join("c_code/exe/predict.c");
    let helper = PathBuf::from(&manifest_dir).join("c_code/helper.c");
    
    eprintln!("=== BUILD CONFIGURATION ===");
    eprintln!("LIEF_LIB: {}", lief_lib);
    eprintln!("LIEF_INCLUDE: {}", lief_include);
    eprintln!("XGBOOST_LIB: {}", xgboost_lib);
    eprintln!("XGBOOST_INCLUDE: {}", xgboost_include);
    eprintln!("OUT_DIR: {}", out_dir.display());
    
    cc::Build::new()
        .file(elf_predict)
        .file(exe_predict)
        .file(helper)
        .include(&lief_include)
        .include(&xgboost_include)
        .compile("predict");

    eprintln!("libpredict.a created at: {}", out_dir.join("libpredict.a").display());
    
    // Rerun if C code changes
    println!("cargo:rerun-if-changed=c_code");
    println!("cargo:rerun-if-changed=c_code/exe/predict.c");
    println!("cargo:rerun-if-changed=c_code/elf/predict.c");
    
    // Set runtime paths
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lief_lib);
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", xgboost_lib);
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lief_wrapper_path);
    
    // Search paths
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-search=native={}", lief_lib);
    println!("cargo:rustc-link-search=native={}", xgboost_lib);
    println!("cargo:rustc-link-search=native={}", lief_wrapper_path);
    
    // Link everything
    // The order matters: dependencies first, then dependents
    println!("cargo:rustc-link-lib=static=LIEF");
    println!("cargo:rustc-link-lib=dylib=xgboost");
    println!("cargo:rustc-link-lib=dylib=stdc++");
    println!("cargo:rustc-link-lib=dylib=lief_wrapper");
    println!("cargo:rustc-link-lib=static=predict");

    let bindings = bindgen::Builder::default()
        .header("./wrapper.h")
        .generate()
        .expect("Couldn't generate the bindings");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write the bindings");
}
