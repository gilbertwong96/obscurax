use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use obscura::Browser;
use rustler::{LocalPid, Resource};
use tokio::sync::{mpsc, oneshot};

use crate::error::ObscuraxError;

pub enum PageCommand {
    Close { reply: oneshot::Sender<()> },
}

pub struct PageHandle {
    pub tx: mpsc::Sender<PageCommand>,
    pub pid: LocalPid,
    pub closed: Arc<AtomicBool>,
}

#[rustler::resource_impl]
impl Resource for PageHandle {}

pub fn spawn_page_thread(
    browser: Arc<Browser>,
    pid: LocalPid,
) -> Result<PageHandle, ObscuraxError> {
    let (tx, rx) = mpsc::channel::<PageCommand>(64);
    let closed = Arc::new(AtomicBool::new(false));
    let closed_clone = closed.clone();

    thread::Builder::new()
        .name("obscurax-page".to_string())
        .spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(_) => {
                    closed_clone.store(true, Ordering::SeqCst);
                    return;
                }
            };
            rt.block_on(async move {
                let mut page = match browser.new_page().await {
                    Ok(p) => p,
                    Err(_) => {
                        closed_clone.store(true, Ordering::SeqCst);
                        return;
                    }
                };
                page_command_loop(&mut page, rx).await;
            });
        })
        .map_err(|e| crate::error::nif_error("internal", format!("spawn page thread: {e}")))?;

    Ok(PageHandle { tx, pid, closed })
}

async fn page_command_loop(page: &mut obscura::Page, mut rx: mpsc::Receiver<PageCommand>) {
    while let Some(cmd) = rx.recv().await {
        match cmd {
            PageCommand::Close { reply } => {
                let _ = reply.send(());
                break;
            }
        }
    }
}
