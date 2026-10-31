use std::path::Path;

use flate2::bufread::GzDecoder;
use tar::Archive;

#[cfg(feature = "packaging-script")]
static MEGATONLIB_TARGZ: &[u8] = &[];
#[cfg(not(feature = "packaging-script"))]
static MEGATONLIB_TARGZ: &[u8] = include_bytes!("../dist/megatonlib.tar.gz");

#[cfg(feature = "packaging-script")]
static MEGATONLIB_TARGZ_SHA256: &str = "";
#[cfg(not(feature = "packaging-script"))]
static MEGATONLIB_TARGZ_SHA256: &str = include_str!("../dist/megatonlib.tar.gz.sha256");

pub fn unpack_megatonlib_if_needed(path: impl AsRef<Path>) -> cu::Result<()> {
    unpack_megatonlib_if_needed_imp(path.as_ref())
}
#[cu::context("failed to unpack megatonlib to '{}'", path.display())]
fn unpack_megatonlib_if_needed_imp(path: &Path) -> cu::Result<()> {
    if MEGATONLIB_TARGZ_SHA256.trim().is_empty() {
        cu::bail!("megatonlib is not included; the buildtool was not built correctly!");
    }
    let sha256_path = path.join(".sha256");
    if let Ok(s) = cu::fs::read_string(&sha256_path) {
        if s.trim() == MEGATONLIB_TARGZ_SHA256.trim() {
            cu::debug!("megatonlib sha256 is up-to-date, skipped unpacking");
            return Ok(());
        }
        cu::debug!("megatonlib sha256 is outdated, unpacking");
    } else {
        cu::debug!("megatonlib sha256 cannot be read, unpacking");
    }

    cu::fs::make_dir_empty(path)?;

    let gz = GzDecoder::new(MEGATONLIB_TARGZ);
    let mut tar = Archive::new(gz);

    cu::check!(tar.unpack(path), "failed to extract megatonlib tar archive")?;
    cu::fs::write(sha256_path, MEGATONLIB_TARGZ_SHA256)?;

    cu::debug!("megatonlib extracted");
    Ok(())
}
