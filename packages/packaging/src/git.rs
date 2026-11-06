use cu::pre::*;

#[cu::context("failed to get current commit")]
pub fn get_commit() -> cu::Result<String> {
    let (child, commit_hash) = cu::which("git")?
        .command()
        .args(["rev-parse", "HEAD"])
        .stdout(cu::pio::string())
        .stdie_null()
        .spawn()?;
    child.wait_nz()?;
    let commit_hash = commit_hash.join()??;
    Ok(commit_hash.trim().to_string())
}
