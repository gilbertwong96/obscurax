use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use obscura::{InterceptResolution, InterceptedRequest};
use rustler::Encoder;
use rustler::LocalPid;
use rustler::OwnedEnv;
use tokio::sync::{mpsc, oneshot};

use crate::atoms;

/// Shared registry of pending interception replies, keyed by intercept id.
///
/// The page-thread interception drain registers a pending reply here (a
/// `oneshot::Sender<InterceptResolution>`) and then waits on the matching
/// receiver. The `reply_intercept` NIF looks the id up and resolves it.
pub struct InterceptRegistry {
    pending: Mutex<HashMap<u64, oneshot::Sender<InterceptResolution>>>,
    counter: AtomicU64,
}

impl InterceptRegistry {
    pub fn new() -> Self {
        Self {
            pending: Mutex::new(HashMap::new()),
            counter: AtomicU64::new(1),
        }
    }

    pub fn next_id(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::Relaxed)
    }

    pub fn register(&self, id: u64, sender: oneshot::Sender<InterceptResolution>) {
        self.pending.lock().unwrap().insert(id, sender);
    }

    pub fn resolve(&self, id: u64, resolution: InterceptResolution) -> bool {
        if let Some(sender) = self.pending.lock().unwrap().remove(&id) {
            sender.send(resolution).is_ok()
        } else {
            false
        }
    }
}

/// Drain the interception channel produced by `Page::enable_interception`.
///
/// For each intercepted request: allocate a fresh intercept id, register a
/// pending reply in `registry`, send `{:obscurax_intercept, id, req_map}` to
/// `pid`, then await the reply (or fail open as `Continue` after 30s).
pub fn spawn_interception_drain(
    mut intercept_rx: mpsc::UnboundedReceiver<InterceptedRequest>,
    pid: LocalPid,
    registry: Arc<InterceptRegistry>,
) {
    tokio::spawn(async move {
        loop {
            let Some(req) = intercept_rx.recv().await else { break };

            let id = registry.next_id();
            let (reply_tx, reply_rx) = oneshot::channel::<InterceptResolution>();
            registry.register(id, reply_tx);

            let req_url = req.url.clone();
            let req_method = req.method.clone();
            let req_resource_type = req.resource_type.clone();

            let mut env = OwnedEnv::new();
            let _ = env.send_and_clear(&pid, |env| {
                let pairs: Vec<(rustler::Term, rustler::Term)> = vec![
                    (atoms::url().encode(env), req_url.encode(env)),
                    (atoms::method().encode(env), req_method.encode(env)),
                    (atoms::resource_type().encode(env), req_resource_type.encode(env)),
                ];
                let req_map = rustler::Term::map_from_pairs(env, &pairs)
                    .unwrap_or(atoms::nil().encode(env));
                (atoms::obscurax_intercept(), id, req_map).encode(env)
            });

            let resolution = match tokio::time::timeout(Duration::from_secs(30), reply_rx).await {
                Ok(Ok(res)) => res,
                _ => InterceptResolution::Continue {
                    url: None,
                    method: None,
                    headers: None,
                    body: None,
                },
            };
            let _ = req.resolver.send(resolution);
        }
    });
}
