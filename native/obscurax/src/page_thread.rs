use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use obscura::Browser;
use rustler::{LocalPid, Resource};
use tokio::sync::{mpsc, oneshot};

use crate::error::ObscuraxError;

pub enum PageCommand {
    Goto {
        url: String,
        reply: oneshot::Sender<Result<(), String>>,
    },
    Url {
        reply: oneshot::Sender<String>,
    },
    Evaluate {
        expr: String,
        reply: oneshot::Sender<serde_json::Value>,
    },
    Content {
        reply: oneshot::Sender<String>,
    },
    QuerySelector {
        selector: String,
        reply: oneshot::Sender<Option<u64>>,
    },
    WaitForSelector {
        selector: String,
        timeout_ms: u64,
        reply: oneshot::Sender<Result<u64, String>>,
    },
    Settle {
        max_ms: u64,
        reply: oneshot::Sender<()>,
    },
    AddPreloadScript {
        script: String,
        reply: oneshot::Sender<()>,
    },
    ElementText {
        node_id: u64,
        reply: oneshot::Sender<String>,
    },
    ElementAttribute {
        node_id: u64,
        name: String,
        reply: oneshot::Sender<Option<String>>,
    },
    ElementClick {
        node_id: u64,
        reply: oneshot::Sender<Result<(), String>>,
    },
    Close {
        reply: oneshot::Sender<()>,
    },
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
            PageCommand::Goto { url, reply } => {
                let res = page.goto(&url).await.map_err(|e| e.to_string());
                let _ = reply.send(res);
            }
            PageCommand::Url { reply } => {
                let _ = reply.send(page.url());
            }
            PageCommand::Evaluate { expr, reply } => {
                let val = page.evaluate(&expr);
                let _ = reply.send(val);
            }
            PageCommand::Content { reply } => {
                let _ = reply.send(page.content());
            }
            PageCommand::QuerySelector { selector, reply } => {
                let elem = page.query_selector(&selector);
                let nid = elem.map(|e| e.node_id());
                let _ = reply.send(nid);
            }
            PageCommand::WaitForSelector { selector, timeout_ms, reply } => {
                let res = page
                    .wait_for_selector(&selector, std::time::Duration::from_millis(timeout_ms))
                    .await
                    .map(|e| e.node_id())
                    .map_err(|e| e.to_string());
                let _ = reply.send(res);
            }
            PageCommand::Settle { max_ms, reply } => {
                page.settle(max_ms).await;
                let _ = reply.send(());
            }
            PageCommand::AddPreloadScript { script, reply } => {
                page.add_preload_script(&script);
                let _ = reply.send(());
            }
            PageCommand::ElementText { node_id, reply } => {
                let js = format!(
                    "(function(){{var el=globalThis._wrap&&globalThis._wrap({});return el?el.textContent:'';}})()",
                    node_id
                );
                let val = page.evaluate(&js);
                let _ = reply.send(val.as_str().unwrap_or("").to_string());
            }
            PageCommand::ElementAttribute { node_id, name, reply } => {
                let js = format!(
                    "(function(){{var el=globalThis._wrap&&globalThis._wrap({});return el?el.getAttribute('{}'):null;}})()",
                    node_id, name
                );
                let val = page.evaluate(&js);
                let result = if val.is_null() {
                    None
                } else {
                    Some(val.as_str().unwrap_or("").to_string())
                };
                let _ = reply.send(result);
            }
            PageCommand::ElementClick { node_id, reply } => {
                let scroll_js = format!(
                    "(function(){{var el=globalThis._wrap&&globalThis._wrap({});if(el)el.scrollIntoView({{block:'center'}});}})()",
                    node_id
                );
                page.evaluate(&scroll_js);
                let click_js = format!(
                    "(function(){{var el=globalThis._wrap&&globalThis._wrap({});if(el){{el.click();return true;}}return false;}})()",
                    node_id
                );
                let val = page.evaluate(&click_js);
                let res = if val.as_bool().unwrap_or(false) {
                    Ok(())
                } else {
                    Err("click failed: element not found".to_string())
                };
                let _ = reply.send(res);
            }
            PageCommand::Close { reply } => {
                let _ = reply.send(());
                break;
            }
        }
    }
}
