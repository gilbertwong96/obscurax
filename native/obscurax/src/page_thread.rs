#![allow(
    clippy::let_and_return,
    clippy::manual_let_else,
    clippy::match_single_binding
)]

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

use obscura_browser::{BrowserContext, Page as InnerPage, WaitUntil};
use obscura_net::{ObscuraHttpClient, RequestCallback, RequestInfo, Response, ResponseCallback};
use rustler::{Encoder, LocalPid, Resource};
use tokio::sync::{mpsc, oneshot};

use crate::atoms;
use crate::callback::InterceptRegistry;
use crate::error::ObscuraxError;

static NEXT_PAGE_ID: AtomicU64 = AtomicU64::new(1);

pub enum PageCommand {
    Goto {
        url: String,
        id: u64,
        pid: rustler::LocalPid,
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

/// Build the context for a single page.
///
/// Every page gets its own `ObscuraHttpClient`, and therefore its own
/// connection pool. Hyper drives each pooled connection with a task on the
/// runtime that created it, and each page's runtime is a short-lived
/// `current_thread` one, so a pool shared across page runtimes would let
/// `Page.close/1` drop a runtime that still backs a sibling page's connection
/// (surfacing as `runtime dropped the dispatch task`). The cookie jar and
/// robots cache stay browser-wide.
fn page_context(base: &BrowserContext, page_id: String) -> Arc<BrowserContext> {
    let mut context = base.isolated_copy(page_id, false);

    let mut client = ObscuraHttpClient::with_full_options(
        base.cookie_jar.clone(),
        base.proxy_url.as_deref(),
        base.allow_private_network,
    );
    client.block_trackers = base.stealth;
    if let Ok(mut user_agent) = client.user_agent.try_write() {
        *user_agent = base.user_agent.clone();
    }

    context.cookie_jar = base.cookie_jar.clone();
    context.robots_cache = base.robots_cache.clone();
    context.http_client = Arc::new(client);
    Arc::new(context)
}

pub fn spawn_page_thread(
    base_context: Arc<BrowserContext>,
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
            // REQUIRED: current_thread, never multi_thread. deno_unsync masks
            // V8's !Send isolate futures as Send and relies on this runtime
            // keeping them on one thread. A multi-thread runtime lets them
            // migrate between workers -- a debug_assert in deno_unsync, but
            // silent unsoundness once the assert is compiled out in release.
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
            // The isolate is built inside the runtime, as the facade's async
            // new_page did, because its construction needs a runtime context.
            rt.block_on(async move {
                let page_id = format!("page-{}", NEXT_PAGE_ID.fetch_add(1, Ordering::Relaxed));
                let context = page_context(&base_context, page_id.clone());
                let mut page = InnerPage::new(page_id, context);
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
    page: &mut InnerPage,
    mut rx: mpsc::Receiver<PageCommand>,
    intercept_registry: Arc<InterceptRegistry>,
) {
    while let Some(cmd) = rx.recv().await {
        match cmd {
            PageCommand::Goto { url, id, pid } => {
                let res = page.navigate_with_wait(&url, WaitUntil::Load).await;
                let mut env = rustler::OwnedEnv::new();
                let _ = env.send_and_clear(&pid, |env| match res {
                    Ok(()) => (atoms::obscurax_result(), id, atoms::ok()).encode(env),
                    Err(e) => {
                        (atoms::obscurax_result(), id, atoms::error(), e.to_string()).encode(env)
                    }
                });
            }
            PageCommand::Url { reply } => {
                let _ = reply.send(page.url_string());
            }
            PageCommand::Evaluate { expr, reply } => {
                let val = page.evaluate(&expr);
                let _ = reply.send(val);
            }
            PageCommand::Content { reply } => {
                let _ = reply.send(page_html(page));
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
                let cb: RequestCallback = std::sync::Arc::new(move |info: &RequestInfo| {
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
                let cb: ResponseCallback =
                    std::sync::Arc::new(move |info: &RequestInfo, resp: &Response| {
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
                    });
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

/// Serialize the live DOM, matching what the facade's `content()` returned.
fn page_html(page: &mut InnerPage) -> String {
    page.evaluate("document.documentElement.outerHTML")
        .as_str()
        .unwrap_or("")
        .to_string()
}

/// Query a DOM node id by CSS selector, mirroring obscura's internal
/// query_selector JS. Returns None if no element matches.
///
/// This inlines the JS that obscura's `Element` wrapper runs so we never
/// touch the `Element` struct (whose `node_id` field is private upstream).
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn query_selector_nid(page: &mut InnerPage, selector: &str) -> Option<u64> {
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
