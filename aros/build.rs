use std::env;
use std::path::{Path, PathBuf};

fn require_file(path: &Path) {
    if !path.is_file() {
        panic!("Required file is missing: {}", path.display());
    }
}

fn main() {
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set"));

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is not set"));

    let target = env::var("TARGET").expect("TARGET is not set by Cargo");

    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS is not set");

    let prefix = manifest_dir.join("vendor").join("RNAlib");
    let include_dir = prefix.join("include");
    let wrapper_header = manifest_dir.join("wrapper.h");

    /*
     * Expected layout:
     *
     * vendor/RNAlib/
     * ├── include/
     * │   └── ViennaRNA/
     * └── prebuilt/
     *     ├── x86_64-apple-darwin/
     *     ├── x86_64-unknown-linux-gnu/
     *     └── x86_64-pc-windows-gnu/
     *         ├── libRNA.a
     *         ├── libgsl.a
     *         ├── libgslcblas.a
     *         ├── libmpfr.a
     *         └── libgmp.a
     *
     * The selected directory is based on Cargo's TARGET value.
     */

    let lib_dir = prefix.join("prebuilt").join(&target);

    if !include_dir.join("ViennaRNA").is_dir() {
        panic!(
            "ViennaRNA headers are missing. Expected directory: {}",
            include_dir.join("ViennaRNA").display(),
        );
    }

    require_file(&wrapper_header);

    if !lib_dir.is_dir() {
        panic!(
            "No ViennaRNA libraries are available for target `{target}`.\n\
             Expected directory: {}\n\
             You must build/download ViennaRNA and all static dependencies \
             for this exact target.",
            lib_dir.display(),
        );
    }

    /*
     * Archive names on macOS/Linux. For a Windows MSVC target, these
     * would normally be .lib files, so handle that target separately
     * when you add Windows support.
     */
    for library in ["RNA", "gsl", "gslcblas", "mpfr", "gmp"] {
        require_file(&lib_dir.join(format!("lib{library}.a")));
    }

    println!("cargo:rerun-if-changed={}", wrapper_header.display());
    println!("cargo:rerun-if-changed={}", include_dir.display());
    println!("cargo:rerun-if-changed={}", lib_dir.display());

    /*
     * Generate bindings from your headers.
     *
     * `--target` tells Clang which C ABI to use. For native builds,
     * such as x86_64 macOS on an Intel Mac, the host SDK provides
     * standard C headers such as stdlib.h.
     */
     let target = std::env::var("TARGET").unwrap();
    // clang doesn't understand Rust's "gnullvm" environment
    let clang_target = target.replace("gnullvm", "gnu");

    let bindings = bindgen::Builder::default()
        .header(
            wrapper_header
                .to_str()
                .expect("wrapper.h path is not valid UTF-8"),
        )
        .clang_arg(format!("-I{}", include_dir.display()))
        .clang_arg(format!("--target={clang_target}"))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        /*
         * Optional but strongly recommended: avoid generating every
         * symbol from every included system/ViennaRNA header.
         *
         * Adjust these allowlists for the ViennaRNA API you use.
         */
        .allowlist_function("vrna_.*")
        .allowlist_type("vrna_.*")
        .allowlist_var("VRNA_.*")
        .generate()
        .expect("Unable to generate ViennaRNA bindings");

    bindings
        .write_to_file(out_dir.join("vienna_rna_bindings.rs"))
        .expect("Could not write generated ViennaRNA bindings");

    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    /*
     * Order matters for static libraries. A later archive resolves
     * unresolved symbols from an earlier archive.
     *
     * RNA -> GSL -> GSL CBLAS -> MPFR -> GMP
     */
    println!("cargo:rustc-link-lib=static=RNA");
    println!("cargo:rustc-link-lib=static=gsl");
    println!("cargo:rustc-link-lib=static=gslcblas");
    println!("cargo:rustc-link-lib=static=mpfr");
    println!("cargo:rustc-link-lib=static=gmp");

    match target_os.as_str() {
        "macos" => {
            // libRNA.a includes C++ objects from libsvm.
            println!("cargo:rustc-link-lib=c++");

            println!("cargo:rustc-link-lib=framework=CoreFoundation");
            println!("cargo:rustc-link-lib=iconv");
            println!("cargo:rustc-link-lib=z");

            /*
             * Add this only if your macOS libRNA.a was built with OpenMP:
             *
             * println!("cargo:rustc-link-lib=omp");
             *
             * Note: libomp is not normally included with macOS itself; it is
             * commonly installed through Homebrew (`brew install libomp`).
             */
        }

        "linux" => {
            // Required by OpenMP references in libRNA.a, e.g. omp_set_dynamic.
            println!("cargo:rustc-link-lib=gomp");

            // Required by C++ libsvm objects embedded in libRNA.a.
            println!("cargo:rustc-link-lib=stdc++");

            println!("cargo:rustc-link-lib=m");
            println!("cargo:rustc-link-lib=pthread");
            println!("cargo:rustc-link-lib=dl");
            println!("cargo:rustc-link-lib=z");
        }

        "windows" => {
            if target.ends_with("gnullvm") {
                // Built with clang in MSYS2 CLANGARM64: LLVM C++ runtime, no libgomp.
                println!("cargo:rustc-link-lib=c++");
                println!("cargo:rustc-link-lib=static=winpthread");
            } else {
                // MinGW GCC (x86_64): GNU C++ runtime.
                println!("cargo:rustc-link-lib=stdc++");
                println!("cargo:rustc-link-lib=gomp");
            }

            println!("cargo:rustc-link-lib=ws2_32");
            println!("cargo:rustc-link-lib=winmm");
        }

        other => panic!("Unsupported target OS `{other}`"),
    }
}
