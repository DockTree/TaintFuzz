extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let frida_sdk = "/usr/games/fuzz/frida/deps/sdk-linux-x86_64";
    
    // Add SDK path FIRST so it takes priority over system libraries
    println!("cargo:rustc-link-search=native={}/lib", frida_sdk);
    
    // Add missing GModule library (this provides g_module_open_full)
    println!("cargo:rustc-link-lib=gmodule-2.0");
    println!("cargo:rustc-link-lib=gobject-2.0");
    println!("cargo:rustc-link-lib=glib-2.0");
    println!("cargo:rustc-link-lib=gio-2.0");

    println!("cargo:rustc-link-lib=pcre2-8");
    
    // System libraries
    println!("cargo:rustc-link-lib=pthread");
    println!("cargo:rustc-link-lib=rt");
    println!("cargo:rustc-link-lib=m");
    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-link-lib=dl");  // Often needed for dynamic loading
    println!("cargo:rustc-link-lib=ffi");

    // Add all required libraries in correct order
    println!("cargo:rustc-link-lib=frida-core");
    println!("cargo:rustc-link-lib=frida-gum-1.0");
    println!("cargo:rustc-link-lib=frida-gumjs-inspector-1.0");
    
    println!("cargo:rerun-if-changed=wrapper.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate bindings for.
        .header("wrapper.h")
        // Add include paths for Frida headers
        .clang_arg("-I/usr/games/fuzz/frida/build/subprojects/frida-core/src/")
        .clang_arg("-I/usr/games/fuzz/frida-gum-devkit")
        .clang_arg("-I/usr/games/fuzz/frida/deps/sdk-linux-x86_64/include/glib-2.0")
        .clang_arg("-I/usr/games/fuzz/frida/deps/sdk-linux-x86_64/lib/glib-2.0/include/")
        .clang_arg("-I/usr/games/fuzz/frida/build/subprojects/frida-core/lib/base/")
        .clang_arg("-I/usr/games/fuzz/frida/deps/sdk-linux-x86_64/include/json-glib-1.0/")
        .clang_arg("-I/usr/games/fuzz/frida/deps/sdk-linux-x86_64/include/nice/")
        .clang_arg("-I/usr/games/fuzz/frida/deps/sdk-linux-x86_64/include/gio-unix-2.0/")
        .clang_arg("-I/usr/games/fuzz/frida/deps/sdk-linux-x86_64/include/gee-0.8/")
        .clang_arg("-I/usr/games/fuzz/frida/subprojects/frida-gum/")
        .clang_arg("-I/usr/games/fuzz/frida/subprojects/frida-gum/mbuild/")
        .clang_arg("-I/usr/games/fuzz/frida/deps/sdk-linux-x86_64/include/capstone/")
        // Generate bindings for functions and types
        .generate_comments(true)
        // Allowlist specific functions/types you want to use
        .allowlist_function("frida_.*")
        .allowlist_type("Frida.*")
        .allowlist_var("FRIDA_.*")
        .allowlist_function("gum_.*")
        .allowlist_type("Gum.*")
        .allowlist_var("GUM_.*")
        // Derive common traits
        .derive_default(true)
        .derive_debug(true)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("frida-bindings.rs"))
        .expect("Couldn't write bindings!");
}