// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Contributors of https://github.com/Pistonite/megaton

use std::fs::File;
use std::path::Path;

use cu::pre::*;
use flate2::{Compression, write::GzEncoder};
use sha2::{Digest, Sha256};
use tar::{Builder as TarBuilder, HeaderMode};

pub struct LibraryPackager {
    /// The tarball builder, holding onto the bytes
    /// since it will not be huge and we need to hash it later
    tar: TarBuilder<GzEncoder<Vec<u8>>>,
}
impl Default for LibraryPackager {
    fn default() -> Self {
        Self::new()
    }
}
impl LibraryPackager {
    pub fn new() -> Self {
        let tar = {
            let gz_encoder = GzEncoder::new(vec![], Compression::default());
            let mut builder = TarBuilder::new(gz_encoder);
            builder.mode(HeaderMode::Deterministic);
            builder.follow_symlinks(false);
            builder
        };
        Self { tar }
    }

    /// Add a file to the path in the archive
    pub fn add_file(
        &mut self,
        file: impl AsRef<Path>,
        archive_path: impl AsRef<Path>,
    ) -> cu::Result<()> {
        let rel_path = archive_path.as_ref();
        cu::ensure!(rel_path.is_relative(), "{}", rel_path.display())?;
        let file_path = file.as_ref();
        let mut file = cu::check!(
            File::open(file_path),
            "failed to open '{}'",
            file_path.display()
        )?;
        cu::debug!("packing {}", rel_path.display());
        self.tar.append_file(rel_path, &mut file)?;
        Ok(())
    }

    /// Add a directory `dir` recursively as `archive_dir` in the archive.
    ///
    /// Follow ignore paths in `.libignore`.
    #[inline(always)]
    pub fn add_dir(
        &mut self,
        dir: impl AsRef<Path>,
        archive_dir: impl AsRef<Path>,
        use_gitignore: bool,
    ) -> cu::Result<()> {
        self.add_dir_imp(dir.as_ref(), archive_dir.as_ref(), use_gitignore)
    }
    fn add_dir_imp(
        &mut self,
        dir: &Path,
        archive_dir: &Path,
        use_gitignore: bool,
    ) -> cu::Result<()> {
        let mut walker = cu::fs::walker(dir);
        walker.git(use_gitignore).add_ignore_filename(".libignore");
        for entry in walker.walk()? {
            let entry = entry?;
            let entry_path = entry.path();
            if !entry_path.is_file() {
                continue;
            }
            let rel_path = archive_dir.join(entry_path.try_to_rel_from(dir));
            cu::ensure!(rel_path.is_relative(), "{}", rel_path.display())?;
            let mut file = cu::check!(
                File::open(entry_path),
                "failed to open '{}'",
                entry_path.display()
            )?;
            cu::debug!("packing {}", rel_path.display());
            self.tar.append_file(&rel_path, &mut file)?;
        }
        Ok(())
    }

    /// Write the output tarball and the sha256 hash as string to files
    pub fn finish(
        self,
        out_path: impl AsRef<Path>,
        out_sha_path: impl AsRef<Path>,
    ) -> cu::Result<LibraryPackageInfo> {
        let gz = self.tar.into_inner()?;
        let buffer = gz.finish()?;
        let sha256 = hash_sha256(&buffer);
        let mut info = LibraryPackageInfo {
            up_to_date: false,
            size: buffer.len() as u64,
            sha256,
        };

        let out_path = out_path.as_ref();
        let out_sha_path = out_sha_path.as_ref();
        if let Ok(existing) = cu::fs::read(out_path) {
            if let Ok(existing_sha) = cu::fs::read_string(out_sha_path) {
                if existing == buffer && info.sha256 == existing_sha {
                    info.up_to_date =true;
                    return Ok(info);
                }
            }
        }

        cu::fs::write(out_path, buffer)?;
        cu::fs::write(out_sha_path, &info.sha256)?;
        Ok(info)
    }
}

pub struct LibraryPackageInfo {
    pub up_to_date: bool,
    pub size: u64,
    pub sha256: String,
}

fn hash_sha256(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(64);
    for b in digest {
        // write to string cannot fail
        let _ = write!(hex, "{b:02x}");
    }
    hex
}
