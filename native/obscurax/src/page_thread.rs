#![allow(
    clippy::let_and_return,
    clippy::manual_let_else,
    clippy::match_single_binding
)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use obscura::Browser;
use rustler::{Encoder, LocalPid, Resource};
use tokio::sync::{mpsc, oneshot};

use crate::atoms;
use crate::callback::InterceptRegistry;
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
    OnRequest {
        callback_id: u64,
        pid: LocalPid,
        reply: oneshot::Sender<()>,
    },
    OnResponse {
        callback_id: u64,
        pid: LocalPid,
        reply: oneshot::Sender<()>,
    },
    OffRequest {
        id: u64,
        reply: oneshot::Sender<bool>,
    },
    OffResponse {
        id: u64,
        reply: oneshot::Sender<bool>,
    },
    EnableInterception {
        pid: LocalPid,
        reply: oneshot::Sender<()>,
    },
    Close {
        reply: oneshot::Sender<()>,
    },
}

pub struct PageHandle {
    pub tx: mpsc::Sender<PageCommand>,
    pub pid: LocalPid,
    /// Set to true when the page thread exits. Reserved for future page_closed detection.
    #[allow(dead_code)]
    pub closed: Arc<AtomicBool>,
    pub intercept_registry: Arc<InterceptRegistry>,
}

#[rustler::resource_impl]
impl Resource for PageHandle {}

pub fn spawn_page_thread(
    browser: Arc<Browser>,
    pid: LocalPid,
) -> Result<PageHandle, Box<ObscuraxError>> {
    let (tx, rx) = mpsc::channel::<PageCommand>(64);
    let closed = Arc::new(AtomicBool::new(false));
    let intercept_registry = Arc::new(InterceptRegistry::new());
    let registry_clone = intercept_registry.clone();
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
                page_command_loop(&mut page, rx, registry_clone).await;
            });
        })
        .map_err(|e| crate::error::nif_error("internal", format!("spawn page thread: {e}")))?;

    Ok(PageHandle {
        tx,
        pid,
        closed,
        intercept_registry,
    })
}

#[allow(clippy::too_many_lines)]
async fn page_command_loop(
    page: &mut obscura::Page,
    mut rx: mpsc::Receiver<PageCommand>,
    intercept_registry: Arc<InterceptRegistry>,
) {
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
                let nid = query_selector_nid(page, &selector);
                let _ = reply.send(nid);
            }
            PageCommand::WaitForSelector {
                selector,
                timeout_ms,
                reply,
            } => {
                let start = std::time::Instant::now();
                let timeout = std::time::Duration::from_millis(timeout_ms);
                let res = loop {
                    if let Some(nid) = query_selector_nid(page, &selector) {
                        break Ok(nid);
                    }
                    if start.elapsed() > timeout {
                        break Err(format!(
                            "wait_for_selector({}) timed out after {}ms",
                            selector, timeout_ms
                        ));
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                };
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
            PageCommand::ElementAttribute {
                node_id,
                name,
                reply,
            } => {
                let escaped_name = name.replace('\\', "\\\\").replace('\'', "\\'");
                let js = format!(
                    "(function(){{var el=globalThis._wrap&&globalThis._wrap({});return el?el.getAttribute('{}'):null;}})()",
                    node_id, escaped_name
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
            PageCommand::OnRequest {
                callback_id,
                pid,
                reply,
            } => {
                let cb: obscura::RequestCallback =
                    std::sync::Arc::new(move |info: &obscura::RequestInfo| {
                        let info_url = info.url.to_string();
                        let info_method = info.method.clone();
                        let info_rt = format!("{:?}", info.resource_type);
                        let mut env = rustler::OwnedEnv::new();
                        let _ = env.send_and_clear(&pid, |env| {
                            let pairs: Vec<(rustler::Term, rustler::Term)> = vec![
                                (atoms::url().encode(env), info_url.encode(env)),
                                (atoms::method().encode(env), info_method.encode(env)),
                                (atoms::resource_type().encode(env), info_rt.encode(env)),
                            ];
                            let req_map = rustler::Term::map_from_pairs(env, &pairs)
                                .unwrap_or(atoms::nil().encode(env));
                            (atoms::obscurax_request(), callback_id, req_map).encode(env)
                        });
                    });
                let _id = page.on_request(cb);
                let _ = reply.send(());
            }
            PageCommand::OnResponse {
                callback_id,
                pid,
                reply,
            } => {
                let cb: obscura::ResponseCallback = std::sync::Arc::new(
                    move |info: &obscura::RequestInfo, resp: &obscura::Response| {
                        let info_url = info.url.to_string();
                        let info_method = info.method.clone();
                        let info_rt = format!("{:?}", info.resource_type);
                        let resp_status = resp.status;
                        let mut env = rustler::OwnedEnv::new();
                        let _ = env.send_and_clear(&pid, |env| {
                            let pairs: Vec<(rustler::Term, rustler::Term)> = vec![
                                (atoms::url().encode(env), info_url.encode(env)),
                                (atoms::method().encode(env), info_method.encode(env)),
                                (atoms::resource_type().encode(env), info_rt.encode(env)),
                                (atoms::status().encode(env), resp_status.encode(env)),
                            ];
                            let msg_map = rustler::Term::map_from_pairs(env, &pairs)
                                .unwrap_or(atoms::nil().encode(env));
                            (atoms::obscurax_response(), callback_id, msg_map).encode(env)
                        });
                    },
                );
                let _id = page.on_response(cb);
                let _ = reply.send(());
            }
            PageCommand::OffRequest { id, reply } => {
                let removed = page.off_request(id);
                let _ = reply.send(removed);
            }
            PageCommand::OffResponse { id, reply } => {
                let removed = page.off_response(id);
                let _ = reply.send(removed);
            }
            PageCommand::EnableInterception { pid, reply } => {
                let intercept_rx = page.enable_interception();
                crate::callback::spawn_interception_drain(
                    intercept_rx,
                    pid,
                    intercept_registry.clone(),
                );
                let _ = reply.send(());
            }
            PageCommand::Close { reply } => {
                let _ = reply.send(());
                break;
            }
        }
    }
}

/// Query a DOM node id by CSS selector, mirroring obscura's internal
/// query_selector JS. Returns None if no element matches.
///
/// This inlines the JS that obscura's `Element` wrapper runs so we never
/// touch the `Element` struct (whose `node_id` field is private upstream).
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn query_selector_nid(page: &mut obscura::Page, selector: &str) -> Option<u64> {
    let escaped = selector.replace('\\', "\\\\").replace('\'', "\\'");
    let js = format!(
        "(function() {{ var el = document.querySelector('{}'); return el ? el._nid : null; }})()",
        escaped
    );
    let val = page.evaluate(&js);
    val.as_u64().or_else(|| {
        val.as_f64()
            .filter(|f| f.is_finite() && *f >= 0.0)
            .map(|f| f as u64)
    })
}
