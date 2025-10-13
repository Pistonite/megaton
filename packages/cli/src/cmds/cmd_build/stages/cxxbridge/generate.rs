use std::path::Path;
use std::process::Command;
use cu::pre::*;

pub struct Generated {
    pub header: Vec<u8>,
    pub cc: Vec<u8>,
}

pub fn run_cxxbridge(rs: &Path) -> cu::Result<Generated> {

    let exe = which::which("cxxbridge").context("cxxbridge not found; `cargo install cxxbridge-cmd`")?;
    let header = Command::new(&exe)
        .arg("--header")
        .arg(rs)
        .output()
        .with_context(|| format!("{} --header {}", exe.display(), rs.display()))?;
    if !header.status.success() {
        cu::bail!("cxxbridge header failed for {}\n{}", rs.display(), String::from_utf8_lossy(&header.stderr));
    }
    let cc = Command::new(&exe)
        .arg(rs)
        .output()
        .with_context(|| format!("{} --cc {}", exe.display(), rs.display()))?;
    if !cc.status.success() {
        cu::bail!("cxxbridge cc failed for {}\n{}", rs.display(), String::from_utf8_lossy(&cc.stderr));
    }
    Ok(Generated { header: header.stdout, cc: cc.stdout })
}

pub fn write_if_changed(path: &Path, bytes: &[u8]) -> cu::Result<bool> {
    use std::fs;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("mkdir -p {}", parent.display()))?;
    }
    let changed = match fs::read(path) { Ok(existing) => existing != bytes, Err(_) => true };
    if changed {
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, bytes).with_context(|| format!("write tmp {}", tmp.display()))?;
        fs::rename(&tmp, path).with_context(|| format!("rename {} -> {}", tmp.display(), path.display()))?;
    }
    Ok(changed)
}
