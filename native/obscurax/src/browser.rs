use std::sync::Arc;

use obscura::Browser;
use rustler::{Encoder, Env, NifResult, Resource, ResourceArc, Term};

use crate::atoms;
use crate::error::nif_error;
use crate::page_thread::spawn_page_thread;

pub struct BrowserHandle {
    pub browser: Arc<Browser>,
}

#[rustler::resource_impl]
impl Resource for BrowserHandle {}

fn map_get_bool(term: Term, key: &str) -> bool {
    term.map_get(key)
        .ok()
        .and_then(|t| t.decode::<bool>().ok())
        .unwrap_or(false)
}

fn map_get_string(term: Term, key: &str) -> Option<String> {
    let val = term.map_get(key).ok()?;
    if val.is_atom() {
        let atom: rustler::Atom = val.decode().ok()?;
        if atom == rustler::types::atom::nil() {
            return None;
        }
    }
    val.decode::<String>().ok()
}

#[rustler::nif]
pub fn browser_new<'a>(env: Env<'a>, opts: Term<'a>) -> NifResult<Term<'a>> {
    let stealth = map_get_bool(opts, "stealth");
    let proxy = map_get_string(opts, "proxy");
    let user_agent = map_get_string(opts, "user_agent");
    let storage_dir = map_get_string(opts, "storage_dir");

    let mut config = obscura::BrowserConfig::default();
    config.stealth = stealth;
    config.proxy = proxy;
    config.user_agent = user_agent;
    if let Some(dir) = storage_dir {
        config.storage_dir = Some(std::path::PathBuf::from(dir));
    }

    match Browser::build(config) {
        Ok(browser) => {
            let handle = ResourceArc::new(BrowserHandle {
                browser: Arc::new(browser),
            });
            Ok((atoms::ok(), handle).encode(env))
        }
        Err(e) => Err(rustler::Error::Term(Box::new(nif_error(
            "internal",
            e.to_string(),
        )))),
    }
}

#[rustler::nif]
pub fn browser_new_page<'a>(
    env: Env<'a>,
    handle: ResourceArc<BrowserHandle>,
    pid: rustler::LocalPid,
) -> NifResult<Term<'a>> {
    match spawn_page_thread(handle.browser.clone(), pid) {
        Ok(page) => {
            let arc = ResourceArc::new(page);
            Ok((atoms::ok(), arc).encode(env))
        }
        Err(e) => Err(rustler::Error::Term(Box::new(e))),
    }
}

#[rustler::nif]
pub fn browser_cookies<'a>(
    env: Env<'a>,
    handle: ResourceArc<BrowserHandle>,
) -> NifResult<Term<'a>> {
    let store = handle.browser.cookies();
    let arc = ResourceArc::new(crate::cookie::CookieStoreHandle::new(store));
    Ok((atoms::ok(), arc).encode(env))
}
