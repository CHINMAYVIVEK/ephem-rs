/*  lib-sys | Rust bindings for lib-swiss, the Swiss Ephemeris C library.
 *  Copyright (c) 2024 Chinmay Vivek. All rights reserved.

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as
    published by the Free Software Foundation, either version 3 of the
    License, or (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

//! # build.rs for lib-sys: Swiss Ephemeris Bindings
//!
//! This build script is responsible for generating Rust bindings for the Swiss Ephemeris C library.
//! It uses the `bindgen` tool to generate the bindings from the C header files, and compiles
//! the required source files from the Swiss Ephemeris library using `cc`.
//!
//! The script also sets appropriate flags for compiling and specifies the necessary
//! libraries for linking.

extern crate bindgen;

use bindgen::callbacks::{MacroParsingBehavior, ParseCallbacks};
use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// A custom callback struct to handle macro parsing behavior during the bindgen process.
///
/// This struct stores a set of macro names that have been encountered during parsing.
/// It also defines specific behavior for certain floating-point macros that should be ignored.
#[derive(Debug)]
struct MacroCallback {
    /// A thread-safe, shared collection of macros encountered during parsing.
    macros: Arc<RwLock<HashSet<String>>>,
}

impl ParseCallbacks for MacroCallback {
    /// Determines the parsing behavior for specific macros.
    ///
    /// Certain floating-point macros (`FP_NAN`, `FP_INFINITE`, etc.) are ignored during parsing
    /// to prevent issues, while others are parsed by default.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the macro being parsed.
    ///
    /// # Returns
    ///
    /// Returns the behavior to either ignore or parse the macro.
    fn will_parse_macro(&self, name: &str) -> MacroParsingBehavior {
        // Insert the macro name into the set of encountered macros.
        self.macros.write().unwrap().insert(name.into());

        // Define specific behavior for known floating-point macros.
        match name {
            "FP_NAN" | "FP_INFINITE" | "FP_ZERO" | "FP_NORMAL" | "FP_SUBNORMAL" => {
                MacroParsingBehavior::Ignore
            }
            _ => MacroParsingBehavior::Default,
        }
    }
}

fn main() {
    // Get the current directory (where the Cargo manifest is located).
    let pwd = env::var("CARGO_MANIFEST_DIR").unwrap();
    let pwd_path = Path::new(&pwd);

    // Get the output directory for the generated bindings.
    let out_path =
        PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR environment variable not set."));

    // Get the path to the Swiss Ephemeris source files, defaulting to the "vendor" directory.
    let libswe_path =
        PathBuf::from(env::var("RUST_LIBSWE_SYS_SOURCE").unwrap_or("vendor".to_owned()));

    // Set the appropriate Clang include argument to point to the Swiss Ephemeris headers.
    let clang_arg = format!("-I{}", libswe_path.to_string_lossy());

    // Compile the Swiss Ephemeris source files using the `cc` crate.
    // The flags `-g`, `-Wall`, and `-fPIC` are set for debugging, enabling all warnings, and position-independent code, respectively.
    cc::Build::new()
        .flag("-g")
        .flag("-Wall")
        .flag("-fPIC")
        .files([
            // Add all the necessary source files for the Swiss Ephemeris.
            pwd_path.join("vendor/swecl.c"),
            pwd_path.join("vendor/swedate.c"),
            pwd_path.join("vendor/swehel.c"),
            pwd_path.join("vendor/swehouse.c"),
            pwd_path.join("vendor/swejpl.c"),
            pwd_path.join("vendor/swemmoon.c"),
            pwd_path.join("vendor/swemplan.c"),
            pwd_path.join("vendor/sweph.c"),
            pwd_path.join("vendor/swephlib.c"),
        ])
        .compile("swe"); // Name the compiled library "swe".

    // Print a directive to rerun the build script if the `wrapper.h` file changes.
    println!("cargo:rerun-if-changed=wrapper.h");

    // Specify the directory where the linker should search for the Swiss Ephemeris library.
    println!("cargo:rustc-link-search={}", libswe_path.to_string_lossy());

    // Link the Swiss Ephemeris library by specifying it to the Rust compiler.
    println!("cargo:rustc-link-lib=swe");

    // Create a shared set of macros for the callback to keep track of encountered macros.
    let macros = Arc::new(RwLock::new(HashSet::new()));

    // Use `bindgen` to generate Rust bindings from the Swiss Ephemeris C headers.
    let bindings = bindgen::Builder::default()
        .header("src/wrapper.h") // Specify the C header file that contains the function declarations.
        .clang_arg(clang_arg) // Provide the Clang include path for the Swiss Ephemeris.
        .parse_callbacks(Box::new(MacroCallback {
            macros: macros.clone(), // Provide the custom macro callback.
        }))
        .allowlist_function("swe_.*") // Allow only functions starting with `swe_` to be included in the bindings.
        .allowlist_var("SE.*") // Allow only variables starting with `SE` to be included in the bindings.
        .generate() // Generate the bindings.
        .expect("Unable to generate bindings."); // Handle any errors that occur during binding generation.

    // Write the generated bindings to the output directory.
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Unable to write bindings to file."); // Handle any errors during the file writing process.
}
