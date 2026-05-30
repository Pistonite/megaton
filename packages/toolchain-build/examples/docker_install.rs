#[cu::cli]
fn main(_: cu::cli::Flags) -> cu::Result<()> {
    megaton_toolchain_build::cmd::install(true /*keep*/, false /*clean*/)
}
