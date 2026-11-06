#[cu::cli]
fn main(cmd: megaton_config::cmd::Cmd) -> cu::Result<()> {
    cmd.run(None)
}
