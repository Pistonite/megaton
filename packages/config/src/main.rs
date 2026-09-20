#[cu::cli]
fn main(cmd: megaton_config::Cmd) -> cu::Result<()> {
    cmd.run(None)
}
