use megaton_packaging as packaging;

fn main() -> cu::Result<()> {
    cu::cli::level("v");
    let md = cu::env_var("CARGO_MANIFEST_DIR").unwrap_or_default();
    if md.is_empty() {
        cu::bail!("failed to get CARGO_MANIFEST_DIR, please run the packaging script with cargo");
    }
    let dist_dir = cu::path!(&md / "lib");
    packaging::unpack_megatonlib_if_needed(&dist_dir)
}
