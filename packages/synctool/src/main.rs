#[cu::cli]
async fn main(cmd: megaton_synctool::cmd::Cmd) -> cu::Result<()> {
    cmd.run_async(None).await
}
