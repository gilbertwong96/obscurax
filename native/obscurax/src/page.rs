use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use rustler::types::tuple::get_tuple;
use rustler::{Atom, Encoder, Env, NifResult, ResourceArc, Term};
use tokio::sync::oneshot;

use crate::atoms::{self, json_to_term};
use crate::error::ObscuraxError;
use crate::page_thread::{PageCommand, PageHandle};

static ASYNC_COUNTER: AtomicU64 = AtomicU64::new(1);

fn page_closed_err() -> rustler::Error {
    rustler::Error::Term(Box::new(ObscuraxError::page_closed()))
}

#[rustler::nif]
pub fn page_goto<'a>(
    env: Env<'a>,
    handle: ResourceArc<PageHandle>,
    url: String,
) -> NifResult<Term<'a>> {
    let id = ASYNC_COUNTER.fetch_add(1, Ordering::Relaxed);
    let (tx, rx) = oneshot::channel();
    if handle
        .tx
        .blocking_send(PageCommand::Goto { url, reply: tx })
        .is_err()
    {
        return Err(rustler::Error::Term(Box::new(ObscuraxError::page_closed())));
    }
    let pid = handle.pid;
    let mut owned_env = rustler::OwnedEnv::new();
    std::thread::spawn(move || {
        let result = rx.blocking_recv();
        let _ = owned_env.send_and_clear(&pid, |env| match result {
            Ok(Ok(())) => (atoms::obscurax_result(), id, atoms::ok()).encode(env),
            Ok(Err(msg)) => (atoms::obscurax_result(), id, atoms::error(), msg).encode(env),
            Err(_) => (atoms::obscurax_result(), id, atoms::error(), "page closed").encode(env),
        });
    });
    Ok(id.encode(env))
}

#[rustler::nif]
pub fn page_url<'a>(env: Env<'a>, handle: ResourceArc<PageHandle>) -> NifResult<Term<'a>> {
    let (tx, rx) = oneshot::channel();
    if handle
        .tx
        .blocking_send(PageCommand::Url { reply: tx })
        .is_err()
    {
        return Err(rustler::Error::Term(Box::new(ObscuraxError::page_closed())));
    }
    let url = rx
        .blocking_recv()
        .map_err(|_| rustler::Error::Term(Box::new(ObscuraxError::page_closed())))?;
    Ok((atoms::ok(), url).encode(env))
}

#[rustler::nif]
pub fn page_close<'a>(env: Env<'a>, handle: ResourceArc<PageHandle>) -> NifResult<Term<'a>> {
    let (tx, rx) = oneshot::channel();
    let _ = handle.tx.blocking_send(PageCommand::Close { reply: tx });
    let _ = rx.blocking_recv();
    Ok(atoms::ok().encode(env))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn page_evaluate<'a>(
    env: Env<'a>,
    handle: ResourceArc<PageHandle>,
    expr: String,
) -> NifResult<Term<'a>> {
    let (tx, rx) = oneshot::channel();
    if handle
        .tx
        .blocking_send(PageCommand::Evaluate { expr, reply: tx })
        .is_err()
    {
        return Err(page_closed_err());
    }
    let val = rx.blocking_recv().map_err(|_| page_closed_err())?;
    Ok((atoms::ok(), json_to_term(env, &val)).encode(env))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn page_content<'a>(
    env: Env<'a>,
    handle: ResourceArc<PageHandle>,
) -> NifResult<Term<'a>> {
    let (tx, rx) = oneshot::channel();
    if handle
        .tx
        .blocking_send(PageCommand::Content { reply: tx })
        .is_err()
    {
        return Err(page_closed_err());
    }
    let html = rx.blocking_recv().map_err(|_| page_closed_err())?;
    Ok((atoms::ok(), html).encode(env))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn page_query_selector<'a>(
    env: Env<'a>,
    handle: ResourceArc<PageHandle>,
    selector: String,
) -> NifResult<Term<'a>> {
    let (tx, rx) = oneshot::channel();
    if handle
        .tx
        .blocking_send(PageCommand::QuerySelector { selector, reply: tx })
        .is_err()
    {
        return Err(page_closed_err());
    }
    let nid = rx.blocking_recv().map_err(|_| page_closed_err())?;
    Ok(match nid {
        Some(id) => (atoms::ok(), id).encode(env),
        None => atoms::nil().encode(env),
    })
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn page_wait_for_selector<'a>(
    env: Env<'a>,
    handle: ResourceArc<PageHandle>,
    selector: String,
    timeout_ms: u64,
) -> NifResult<Term<'a>> {
    let (tx, rx) = oneshot::channel();
    if handle
        .tx
        .blocking_send(PageCommand::WaitForSelector {
            selector,
            timeout_ms,
            reply: tx,
        })
        .is_err()
    {
        return Err(page_closed_err());
    }
    match rx.blocking_recv() {
        Ok(Ok(id)) => Ok((atoms::ok(), id).encode(env)),
        Ok(Err(msg)) => Err(rustler::Error::Term(Box::new(
            crate::error::nif_error("timeout", msg),
        ))),
        Err(_) => Err(page_closed_err()),
    }
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn page_settle<'a>(
    env: Env<'a>,
    handle: ResourceArc<PageHandle>,
    max_ms: u64,
) -> NifResult<Term<'a>> {
    let (tx, rx) = oneshot::channel();
    if handle
        .tx
        .blocking_send(PageCommand::Settle { max_ms, reply: tx })
        .is_err()
    {
        return Err(page_closed_err());
    }
    let _ = rx.blocking_recv();
    Ok(atoms::ok().encode(env))
}

#[rustler::nif]
pub fn page_add_preload_script<'a>(
    env: Env<'a>,
    handle: ResourceArc<PageHandle>,
    script: String,
) -> NifResult<Term<'a>> {
    let (tx, rx) = oneshot::channel();
    if handle
        .tx
        .blocking_send(PageCommand::AddPreloadScript { script, reply: tx })
        .is_err()
    {
        return Err(page_closed_err());
    }
    let _ = rx.blocking_recv();
    Ok(atoms::ok().encode(env))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn page_element_text<'a>(
    env: Env<'a>,
    handle: ResourceArc<PageHandle>,
    node_id: u64,
) -> NifResult<Term<'a>> {
    let (tx, rx) = oneshot::channel();
    if handle
        .tx
        .blocking_send(PageCommand::ElementText { node_id, reply: tx })
        .is_err()
    {
        return Err(page_closed_err());
    }
    let text = rx.blocking_recv().map_err(|_| page_closed_err())?;
    Ok((atoms::ok(), text).encode(env))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn page_element_attribute<'a>(
    env: Env<'a>,
    handle: ResourceArc<PageHandle>,
    node_id: u64,
    name: String,
) -> NifResult<Term<'a>> {
    let (tx, rx) = oneshot::channel();
    if handle
        .tx
        .blocking_send(PageCommand::ElementAttribute {
            node_id,
            name,
            reply: tx,
        })
        .is_err()
    {
        return Err(page_closed_err());
    }
    let attr = rx.blocking_recv().map_err(|_| page_closed_err())?;
    Ok(match attr {
        Some(v) => (atoms::ok(), v).encode(env),
        None => atoms::nil().encode(env),
    })
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn page_element_click<'a>(
    env: Env<'a>,
    handle: ResourceArc<PageHandle>,
    node_id: u64,
) -> NifResult<Term<'a>> {
    let (tx, rx) = oneshot::channel();
    if handle
        .tx
        .blocking_send(PageCommand::ElementClick { node_id, reply: tx })
        .is_err()
    {
        return Err(page_closed_err());
    }
    match rx.blocking_recv() {
        Ok(Ok(())) => Ok(atoms::ok().encode(env)),
        Ok(Err(msg)) => Err(rustler::Error::Term(Box::new(
            crate::error::nif_error("element_not_found", msg),
        ))),
        Err(_) => Err(page_closed_err()),
    }
}

#[rustler::nif]
pub fn page_on_request<'a>(
    env: Env<'a>,
    handle: ResourceArc<PageHandle>,
    callback_id: u64,
    callback_pid: rustler::LocalPid,
) -> NifResult<Term<'a>> {
    let (tx, rx) = oneshot::channel();
    if handle
        .tx
        .blocking_send(PageCommand::OnRequest {
            callback_id,
            pid: callback_pid,
            reply: tx,
        })
        .is_err()
    {
        return Err(page_closed_err());
    }
    let _ = rx.blocking_recv();
    Ok(atoms::ok().encode(env))
}

#[rustler::nif]
pub fn page_on_response<'a>(
    env: Env<'a>,
    handle: ResourceArc<PageHandle>,
    callback_id: u64,
    callback_pid: rustler::LocalPid,
) -> NifResult<Term<'a>> {
    let (tx, rx) = oneshot::channel();
    if handle
        .tx
        .blocking_send(PageCommand::OnResponse {
            callback_id,
            pid: callback_pid,
            reply: tx,
        })
        .is_err()
    {
        return Err(page_closed_err());
    }
    let _ = rx.blocking_recv();
    Ok(atoms::ok().encode(env))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn page_off_request<'a>(
    env: Env<'a>,
    handle: ResourceArc<PageHandle>,
    id: u64,
) -> NifResult<Term<'a>> {
    let (tx, rx) = oneshot::channel();
    if handle
        .tx
        .blocking_send(PageCommand::OffRequest { id, reply: tx })
        .is_err()
    {
        return Err(page_closed_err());
    }
    let removed = rx.blocking_recv().unwrap_or(false);
    Ok(removed.encode(env))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn page_off_response<'a>(
    env: Env<'a>,
    handle: ResourceArc<PageHandle>,
    id: u64,
) -> NifResult<Term<'a>> {
    let (tx, rx) = oneshot::channel();
    if handle
        .tx
        .blocking_send(PageCommand::OffResponse { id, reply: tx })
        .is_err()
    {
        return Err(page_closed_err());
    }
    let removed = rx.blocking_recv().unwrap_or(false);
    Ok(removed.encode(env))
}

#[rustler::nif]
pub fn page_enable_interception<'a>(
    env: Env<'a>,
    handle: ResourceArc<PageHandle>,
    callback_pid: rustler::LocalPid,
) -> NifResult<Term<'a>> {
    let (tx, rx) = oneshot::channel();
    if handle
        .tx
        .blocking_send(PageCommand::EnableInterception {
            pid: callback_pid,
            reply: tx,
        })
        .is_err()
    {
        return Err(page_closed_err());
    }
    let _ = rx.blocking_recv();
    Ok(atoms::ok().encode(env))
}

#[rustler::nif]
pub fn reply_intercept<'a>(
    env: Env<'a>,
    handle: ResourceArc<PageHandle>,
    intercept_id: u64,
    decision: Term,
) -> NifResult<Term<'a>> {
    let resolution = decode_intercept_decision(decision)?;
    let _delivered = handle.intercept_registry.resolve(intercept_id, resolution);
    Ok(atoms::ok().encode(env))
}

fn decode_intercept_decision(term: Term) -> NifResult<obscura::InterceptResolution> {
    if let Ok(atom) = Atom::from_term(term) {
        if atom == atoms::continue_() {
            return Ok(obscura::InterceptResolution::Continue {
                url: None,
                method: None,
                headers: None,
                body: None,
            });
        }
    }

    let tuple_data = get_tuple(term)?;
    if tuple_data.is_empty() {
        return Err(rustler::Error::RaiseAtom("invalid_intercept_decision"));
    }
    let tag = Atom::from_term(tuple_data[0])?;
    if tag == atoms::fulfill() {
        let status: u16 = tuple_data[1].decode()?;
        let headers: HashMap<String, String> = tuple_data[2].decode()?;
        let body: String = tuple_data[3].decode()?;
        return Ok(obscura::InterceptResolution::Fulfill {
            status,
            headers,
            body,
        });
    }
    if tag == atoms::fail() {
        let reason: String = tuple_data[1].decode()?;
        return Ok(obscura::InterceptResolution::Fail { reason });
    }

    Err(rustler::Error::RaiseAtom("invalid_intercept_decision"))
}
