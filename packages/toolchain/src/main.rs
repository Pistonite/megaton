#[cu::cli]
fn main(cmd: megaton_toolchain::cmd::Cmd) -> cu::Result<()> {
    cmd.run()
}
