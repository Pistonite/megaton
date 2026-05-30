// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Megaton contributors

use std::path::Path;

use cu::pre::*;

use flate2::bufread::GzDecoder;


static LIBRARY_TARGZ: &[u8] = include_bytes!("../../libmegaton.tar.gz");

pub async fn unpack_megaton_lib(lib_path: &Path) -> cu::Result<()> {
    if needs_unpack(lib_path).await {
        cu::trace!("unpacking megaton lib");
        do_unpack(lib_path).await?;
        cu::debug!("done unpacking megaton lib");
    } else {
        cu::debug!("megaton lib is up-to-date");
    }
    Ok(())
}

/// True if version hash doesn't exist or doesn't match the stored tarball
async fn needs_unpack(lib_path: &Path) -> bool {
    let lib_hash_file = lib_path.join(".hash");
    if !lib_hash_file.exists() {
        return true;
    }
    let Ok(existing_lib_hash) = cu::fs::co_read(lib_hash_file).await else {
        return true;
    };

    env!("MEGATON_LIB_SHA256").as_bytes() != existing_lib_hash
}

/// Deletes contents of dir and unpacks the library from the stored tarball
async fn do_unpack(lib_path: &Path) -> cu::Result<()> {
    let library_tar = GzDecoder::new(LIBRARY_TARGZ);
    let mut library_archive = tar::Archive::new(library_tar);
    cu::fs::co_make_dir_empty(lib_path).await?;
    let lib_hash_file = lib_path.join(".hash");
    let lib_path = lib_path.to_owned();
    let handle = cu::co::spawn_blocking(move || {
        library_archive.unpack(lib_path)?;
        cu::Ok(())
    });
    cu::check!(handle.co_join().await.flatten(), "failed to unpack megaton library")?;
    cu::fs::co_write(lib_hash_file, env!("MEGATON_LIB_SHA256").as_bytes()).await?;
    
    Ok(())
}
