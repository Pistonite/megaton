#[cu::cli]
fn main(_: cu::cli::Flags) -> cu::Result<()> {
    megaton_docker_setup::setup_isolated_crates(
        "dist-image",
        &[
            "cli",
            "buildtool",
            "config",
            "synctool",
            "toolchain",
            "packaging",
        ],
    )
}
