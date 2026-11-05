#[cu::cli]
fn main(cmd: megaton_buildtool::cmd::Cmd) -> cu::Result<()> {
    cmd.run(None)
}
