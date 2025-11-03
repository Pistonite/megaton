// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Megaton contributors

use std::path::Path;

use cu::pre::*;

use super::RustCrate;


pub fn generate_cxx_bridge_src(rust_crate: RustCrate, module_target_path: impl AsRef<Path>) -> cu::Result<()> {
    // TODO: Parse rust crate for cxxbridge files 
    //
    // TODO: Place generated headers in {module}/include/rust/
    //
    // TODO: Place generated source in {module}/src/cxxbridge

    Ok(())
}
