use std::path::PathBuf;
use std::sync::Arc;

use obscura_browser::BrowserContext;
use rustler::{Atom, Encoder, Env, NifResult, Resource, ResourceArc, Term};

use crate::atoms;
use crate::page_thread::spawn_page_thread;

/// A launched browser. Holds the base context only as a template: each page
/// clones it into its own context so no two page runtimes share an HTTP
/// connection pool.
pub struct BrowserHandle {
    pub context: Arc<BrowserContext>,
}

#[rustler::resource_impl]
impl Resource for BrowserHandle {}

// Keys must be atoms. `Term::map_get/2` encodes the key it is given, and a
// `&str` encodes as a binary, which never matches the atom keys the Elixir side
// sends — so a `&str` key silently reads as "absent".
fn map_get_bool(term: Term, key: Atom) -> bool {
    term.map_get(key)
        .ok()
        .and_then(|t| t.decode::<bool>().ok())
        .unwrap_or(false)
}

fn map_get_string(term: Term, key: Atom) -> Option<String> {
    let val = term.map_get(key).ok()?;
    if val.is_atom() {
        let atom: rustler::Atom = val.decode().ok()?;
        if atom == rustler::types::atom::nil() {
            return None;
        }
    }
    val.decode::<String>().ok()
}

// Building the context sets up the cookie jar, robots cache and TLS/env config,
// so keep it off a normal BEAM scheduler.
#[rustler::nif(schedule = "DirtyCpu")]
pub fn browser_new<'a>(env: Env<'a>, opts: Term<'a>) -> NifResult<Term<'a>> {
    let stealth = map_get_bool(opts, atoms::stealth());
    let proxy = map_get_string(opts, atoms::proxy());
    let user_agent = map_get_string(opts, atoms::user_agent());
    let storage_dir = map_get_string(opts, atoms::storage_dir());
    let allow_file_access = map_get_bool(opts, atoms::allow_file_access());

    let mut context = match storage_dir {
        Some(dir) => BrowserContext::with_storage_full(
            "obscurax".to_string(),
            proxy,
            stealth,
            user_agent,
            Some(PathBuf::from(dir)),
        ),
        None => {
            BrowserContext::with_full_options("obscurax".to_string(), proxy, stealth, user_agent)
        }
    };

    // No obscura constructor accepts this flag, and pages inherit it through
    // `isolated_copy`, so setting it here covers the whole browser.
    context.allow_file_access = allow_file_access;

    let handle = ResourceArc::new(BrowserHandle {
        context: Arc::new(context),
    });
    Ok((atoms::ok(), handle).encode(env))
}

#[rustler::nif]
pub fn browser_new_page<'a>(
    env: Env<'a>,
    handle: ResourceArc<BrowserHandle>,
    pid: rustler::LocalPid,
) -> NifResult<Term<'a>> {
    match spawn_page_thread(handle.context.clone(), pid) {
        Ok(page) => {
            let arc = ResourceArc::new(page);
            Ok((atoms::ok(), arc).encode(env))
        }
        Err(e) => Err(rustler::Error::Term(e)),
    }
}

#[rustler::nif]
pub fn browser_cookies<'a>(
    env: Env<'a>,
    handle: ResourceArc<BrowserHandle>,
) -> NifResult<Term<'a>> {
    let arc = ResourceArc::new(crate::cookie::CookieStoreHandle::new(
        handle.context.cookie_jar.clone(),
    ));
    Ok((atoms::ok(), arc).encode(env))
}
