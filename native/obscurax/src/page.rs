use std::sync::atomic::{AtomicU64, Ordering};

use rustler::{Encoder, Env, NifResult, ResourceArc, Term};
use tokio::sync::oneshot;

use crate::atoms;
use crate::error::ObscuraxError;
use crate::page_thread::{PageCommand, PageHandle};

static ASYNC_COUNTER: AtomicU64 = AtomicU64::new(1);

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
