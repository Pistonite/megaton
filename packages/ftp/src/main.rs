#[cu::cli]
async fn main(cmd: megaton_ftp::cmd::Cmd) -> cu::Result<()> {
    cmd.run(None).await
}
