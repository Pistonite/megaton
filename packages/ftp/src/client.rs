use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use cu::pre::*;

use suppaftp::tokio::{AsyncDataStream, AsyncFtpStream, AsyncNoTlsStream};
use suppaftp::types::Response;
use suppaftp::{FtpError, Status};

type DataStream = AsyncDataStream<AsyncNoTlsStream>;

macro_rules! execute_with_retries {
    (
        #[signal($ctrlc:expr)]
        #[retries($retries:expr)]
        #[on_attempt($on_attempt:expr)]
        #[on_failure($on_failure:expr)]
        #[delay($($delay:tt)*)]
        $execute:expr
    ) => {{
        let fn_on_attempt = $on_attempt;
        let fn_on_failure = cast_fn_failure($on_failure);
        let fn_delay = execute_with_retries!(__make_fn $($delay)*);
        match $execute{
            Ok(x) => Ok(x),
            Err(e1) => {
                fn_on_failure(0, &e1);
                let mut result = Err(e1);
                for i in 1..=$retries {
                    let delay = fn_delay(i);
                    {
                        let bar = cu::progress("waiting before retry")
                            .total(delay as usize)
                            .keep(false)
                            .eta(false)
                            .percentage(false)
                            .spawn();
                        for _ in 0..delay {
                            for _ in 0..5 {
                                tokio::time::sleep(Duration::from_millis(200)).await;
                                $ctrlc.check()?;
                            }
                            cu::progress!(bar += 1);
                        }
                        bar.done();
                    }
                    fn_on_attempt(i);
                    match $execute{
                        Ok(x) => { result = Ok(x); break; }
                        Err(e) => { fn_on_failure(i, &e); result = Err(e); }
                    }
                }
                cu::check!(result, "maximum number of retries reached")
            }
        }
    }};
    (__make_fn |$x:ident| $($expr:tt)*) => {|$x|$($expr)*};
    (__make_fn $($expr:tt)*) => {|_|$($expr)*};
}
// bruh (needed to infer the correct type in the macro)
fn cast_fn_failure<T: Fn(u32, &FtpError)>(f: T) -> T {
    f
}

pub struct FtpClient {
    server: String,
    stream: AsyncFtpStream,
}

impl FtpClient {
    pub async fn connect(server: &str) -> cu::Result<Self> {
        let result = execute_in_ctrlc_frame(move |ctrlc| async move {
            execute_with_retries! {
                #[signal(ctrlc)]
                #[retries(3)]
                #[on_attempt(|i| cu::info!("connecting to {server}... (attempt #{})", i + 1))]
                #[on_failure(|_, e| cu::error!("failed to connect to server: {e}"))]
                #[delay(|i| 2u32.pow(i))]
                AsyncFtpStream::connect(server).await
            }
        })
        .await?;
        let stream = cu::check!(result, "connection cancelled")?;
        cu::info!("connected to {server}");
        Ok(Self {
            server: server.to_string(),
            stream,
        })
    }

    pub async fn quit(mut self) {
        if let Err(e) = self.stream.quit().await {
            cu::warn!("error whiling closing the conection: {e}");
        }
    }

    /// Upload multiple files to the same directory. Create the directory if does not exist
    pub async fn upload(&mut self, dir: &str, files: Vec<(String, Vec<u8>)>) -> cu::Result<()> {
        let result = execute_in_ctrlc_frame(move |ctrlc| async move {
            // change/create the directory first
            cu::info!("cwd: {dir}");
            execute_with_retries! {
                #[signal(ctrlc)]
                #[retries(2)]
                #[on_attempt(|i| cu::info!("cwd: {dir} (attempt #{i})", i=i+1))]
                #[on_failure(|_, e| cu::warn!("failed to change directory: {e}"))]
                #[delay(2)]
                match self.stream.cwd(dir).await {
                    Err(_) => {
                        ctrlc.check()?;
                        // try creating each parent...
                        let path = Path::new(dir);
                        let mut path_builder = PathBuf::from("/");
                        for s in path.components() {
                            use std::path::Component;
                            match s {
                                Component::Prefix(_) => {}
                                Component::RootDir => {},
                                Component::CurDir => {}
                                Component::ParentDir => { path_builder.pop();}
                                Component::Normal(s) => {
                                    path_builder.push(s);
                                    // unwrap: since dir was a string, each component must also
                                    // be valid utf-8
                                    let p = path_builder.as_utf8().unwrap();
                                    let _:Result<_,_> = self.stream.mkdir(p).await;
                                }
                            }
                        }
                        self.stream.cwd(dir).await
                    }
                    Ok(()) => Ok(())
                }
            }?;

            // put each file
            // right now, each file is small, so we do them in serial
            for (file, bytes) in files {
                ctrlc.check()?;
                cu::info!("stor: {file}");
                execute_with_retries! {
                    #[signal(ctrlc)]
                    #[retries(2)]
                    #[on_attempt(|i| cu::info!("stor: {file} (attempt #{i})", i=i+1))]
                    #[on_failure(|_, e| cu::warn!("failed to stor: {file}: {e}"))]
                    #[delay(2)]
                    self.stream.put_file(&file, &mut Cursor::new(&bytes)).await
                }?;
            }

            Ok(())
        })
        .await?;
        if result.is_none() {
            cu::bail!("upload cancelled");
        }
        Ok(())
    }

    pub async fn download(&mut self, dir: &str, out_root: &str, keep: bool) -> cu::Result<()> {
        cu::info!("downloading {dir}");
        let result = execute_in_ctrlc_frame(move |ctrlc| async move {
            let mut pending_dirs = vec!["".to_string()];
            let download_pool = cu::co::pool(4);
            let mut download_handles = cu::co::set([]);
            let mut active_handle_count = 0;
            let mut success_count = 0;
            let mut error_count = 0;
            loop {
                ctrlc.check()?;
                let Some(rel_dir) = pending_dirs.pop() else {
                    break;
                };
                let full_path = if rel_dir.is_empty() {
                    dir.to_string()
                } else {
                    format!("{dir}/{rel_dir}")
                };
                let listing = execute_with_retries! {
                    #[signal(ctrlc)]
                    #[retries(2)]
                    #[on_attempt(|i| cu::info!("mlsd: {full_path} (attempt #{i})", i=i+1))]
                    #[on_failure(|_, e| cu::warn!("failed to list directory: {e}"))]
                    #[delay(2)]
                    match self.stream.mlsd(Some(&full_path)).await {
                        Err(FtpError::UnexpectedResponse(Response {status:
                            Status::FileUnavailable
                            ,..}))
                        => {
                            Ok(None)
                        }
                        Err(e) => Err(e),
                        Ok(x) => Ok(Some(x)),
                    }
                };
                let listing = match listing {
                    Ok(None) => {
                        cu::warn!("skipped {full_path}: directory does not exist or unavailable");
                        continue;
                    }
                    Ok(Some(x)) => x,
                    Err(e) => {
                        cu::error!("skipped {full_path}: error listing directory: {e}");
                        continue;
                    }
                };
                for e in listing.iter().filter_map(|x| Entry::filter_parse(x)) {
                    let rel_path_next = if rel_dir.is_empty() {
                        e.name.to_string()
                    } else {
                        format!("{rel_dir}/{e}", e = e.name)
                    };
                    if e.is_dir {
                        pending_dirs.push(rel_path_next);
                        continue;
                    }
                    let ctrlc = ctrlc.clone();
                    let server_path = format!("{full_path}/{e}", e = e.name);
                    let local_path = Path::new(out_root).join(&rel_path_next);
                    let size = e.size;
                    let retr_client = RetrClient {
                        ctrlc,
                        server: self.server.clone(),
                        server_path,
                        local_path,
                        size,
                        keep,
                    };
                    let handle =
                        download_pool.spawn(async move { retr_client.do_download().await });
                    download_handles.add(handle);
                    active_handle_count += 1;
                }

                // if there are too many handles, drain some first
                if active_handle_count > 128 {
                    for _ in 0..64 {
                        let Some(result) = download_handles.next().await else {
                            break;
                        };
                        match result.flatten() {
                            Err(e) => {
                                cu::error!("error: {e:?}");
                                error_count += 1;
                            }
                            Ok(()) => {
                                success_count += 1;
                            }
                        }
                    }
                }
            }

            while let Some(result) = download_handles.next().await {
                match result.flatten() {
                    Err(e) => {
                        cu::error!("error: {e:?}");
                        error_count += 1;
                    }
                    Ok(()) => {
                        success_count += 1;
                    }
                }
            }

            cu::info!("{success_count} files downloaded from {dir}");
            if error_count > 0 {
                cu::warn!("{error_count} files failed to download!");
                cu::bail!("failed to download some files from {dir}");
            }

            Ok(())
        })
        .await?;
        if result.is_none() {
            cu::bail!("download cancelled");
        }
        Ok(())
    }
}

async fn execute_in_ctrlc_frame<T, TFuture, F>(f: F) -> cu::Result<Option<T>>
where
    TFuture: Future<Output = cu::Result<T>>,
    F: FnOnce(cu::CtrlcSignal) -> TFuture,
{
    let stop_requested = AtomicBool::new(false);
    cu::cli::ctrlc_frame()
        .on_signal(move |_| {
            if stop_requested
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                cu::hint!("graceful stop requested; Ctrl-C again to terminate");
                return;
            }
            cu::error!("forceful termination requested");
            std::process::exit(1);
        })
        .co_execute(f)
        .await
}

struct Entry<'a> {
    is_dir: bool,
    size: Option<u64>,
    name: &'a str,
}
impl<'a> Entry<'a> {
    fn filter_parse(entry: &'a str) -> Option<Self> {
        let mut last = entry;
        let mut is_dir = None;
        let mut size = None;
        for part in entry.split(';') {
            if let Some(ty) = part.strip_prefix("Type=") {
                let ty = ty.trim();
                match ty {
                    "dir" => is_dir = Some(true),
                    "file" => is_dir = Some(false),
                    _ => {}
                }
                continue;
            }
            if let Some(s) = part.strip_prefix("Size=") {
                if let Ok(s) = cu::parse::<u64>(s.trim()) {
                    size = Some(s);
                }
                continue;
            }
            last = part;
        }
        // file name should be last
        Some(Self {
            is_dir: is_dir?,
            size,
            name: last.trim(),
        })
    }
}

struct RetrClient {
    ctrlc: cu::CtrlcSignal,
    server: String,
    server_path: String,
    local_path: PathBuf,
    size: Option<u64>,
    keep: bool,
}
impl RetrClient {
    async fn do_download(self) -> cu::Result<()> {
        let ctrlc = self.ctrlc;
        let server = self.server;
        let mut stream = execute_with_retries! {
            #[signal(ctrlc)]
            #[retries(2)]
            #[on_attempt(|i| cu::info!("connect {server} (attempt #{i})", i=i+1))]
            #[on_failure(|_, e| cu::warn!("failed to open connection: {e}"))]
            #[delay(2)]
            AsyncFtpStream::connect(&server).await
        }?;
        ctrlc.check()?;
        let server_path = self.server_path;
        let data_stream = execute_with_retries! {
            #[signal(ctrlc)]
            #[retries(2)]
            #[on_attempt(|i| cu::info!("retr: {server_path} (attempt #{i})", i=i+1))]
            #[on_failure(|_, e| cu::warn!("failed to retr file: {e}"))]
            #[delay(2)]
            match stream.retr_as_stream(&server_path).await {
                Err(FtpError::UnexpectedResponse(Response {status:
                    Status::FileUnavailable
                    ,..}))
                => {
                    Ok(None)
                }
                Err(e) => Err(e),
                Ok(x) => Ok(Some(x)),
            }
        };
        let mut data_stream = match data_stream {
            Ok(None) => {
                cu::bail!("file unavailable: {server_path}");
            }
            Ok(Some(x)) => x,
            Err(e) => {
                cu::bail!("error downloading file {server_path}: {e}");
            }
        };

        let result = Self::download_with_stream(
            &mut data_stream,
            &ctrlc,
            &server_path,
            self.size,
            self.local_path,
        )
        .await;
        if let Err(e) = stream.finalize_retr_stream(data_stream).await {
            cu::warn!("failed to finalize_retr_stream: {e}");
        }
        if !self.keep && result.is_ok() {
            let result = execute_with_retries! {
                #[signal(ctrlc)]
                #[retries(2)]
                #[on_attempt(|i| cu::info!("rm: {server_path} (attempt #{i})", i=i+1))]
                #[on_failure(|_, e| cu::warn!("failed to remove file: {e}"))]
                #[delay(2)]
                match stream.rm(&server_path).await {
                    Err(FtpError::UnexpectedResponse(Response {status:
                        Status::FileUnavailable
                        ,..}))
                    => {
                        Ok(false)
                    }
                    Err(e) => Err(e),
                    Ok(()) => Ok(true),
                }
            };
            match result {
                Ok(false) => {
                    cu::warn!(
                        "failed to remove {server_path} after download: remote file is unavailable"
                    );
                }
                Err(e) => {
                    cu::warn!("failed to remove {server_path} after download: {e}");
                }
                Ok(true) => {
                    cu::info!("removed {server_path}");
                }
            }
        }

        if let Err(e) = stream.quit().await {
            cu::warn!("failed to close connection: {e}");
        }
        if let Err(e) = result {
            cu::bail!("failed download file: {e:?}");
        }

        Ok(())
    }

    async fn download_with_stream(
        reader: &mut DataStream,
        ctrlc: &cu::CtrlcSignal,
        server_path: &str,
        size: Option<u64>,
        local_path: PathBuf,
    ) -> cu::Result<()> {
        if let Some(dir) = local_path.parent() {
            cu::fs::co_make_dir(dir).await?;
        }
        let ui_path = truncate_path_for_ui(server_path);
        if size == Some(0) {
            cu::fs::co_write(local_path, &[]).await?;
            cu::hint!("written 0-byte file {ui_path}");
            return Ok(());
        }
        let bar = match size {
            None => {
                let bar = cu::progress(format!("download {ui_path}"))
                    .eta(false)
                    .percentage(false)
                    .spawn();
                cu::progress!(bar, "unknown size");
                Some(bar)
            }
            Some(size) => {
                let path = truncate_path_for_ui(server_path);
                let bar = cu::progress(format!("retr: {path}"))
                    .total_bytes(size)
                    .eta(false)
                    .percentage(false)
                    .spawn();
                Some(bar)
            }
        };
        let mut writer = cu::fs::writer(&local_path)?;
        let mut buf = vec![0u8; 8192].into_boxed_slice();
        loop {
            let len = match reader.read(&mut buf).await {
                Ok(0) => break,
                Ok(x) => x,
                Err(e) => {
                    cu::rethrow!(e, "failed to read from download stream");
                }
            };
            if let Some(bar) = &bar {
                cu::progress!(bar += len);
            }
            ctrlc.check()?;
            if let Err(e) = writer.write_all(&buf[0..len]) {
                cu::rethrow!(
                    e,
                    "failed to write to file '{}': io error",
                    local_path.display()
                );
            }
        }
        cu::check!(
            writer.flush(),
            "failed to finalize file '{}'",
            local_path.display()
        )?;
        if let Some(bar) = bar {
            bar.done();
        }
        Ok(())
    }
}

fn truncate_path_for_ui(path: &str) -> String {
    let mut i = path.len();
    for _ in 0..3 {
        let Some(j) = path[0..i].rfind('/') else {
            return path.to_string();
        };
        i = j;
    }
    if i == 0 {
        return path.to_string();
    }
    format!("...{}", &path[i..])
}
