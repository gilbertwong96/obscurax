use std::path::Path;

use obscura::CookieStore;
use rustler::{Encoder, Env, NifResult, Resource, ResourceArc, Term};

use crate::atoms;
use crate::error::ObscuraxError;

pub struct CookieStoreHandle {
    pub store: CookieStore,
}

#[rustler::resource_impl]
impl Resource for CookieStoreHandle {}

impl CookieStoreHandle {
    pub fn new(store: CookieStore) -> Self {
        Self { store }
    }
}

fn cookie_to_term<'a>(env: Env<'a>, c: &obscura::Cookie) -> Term<'a> {
    let pairs: Vec<(Term, Term)> = vec![
        (atoms::name().encode(env), c.name.encode(env)),
        (atoms::value().encode(env), c.value.encode(env)),
        (atoms::domain().encode(env), c.domain.encode(env)),
        (atoms::path().encode(env), c.path.encode(env)),
        (atoms::secure().encode(env), c.secure.encode(env)),
        (atoms::http_only().encode(env), c.http_only.encode(env)),
    ];
    rustler::Term::map_from_pairs(env, &pairs).unwrap_or(atoms::nil().encode(env))
}

#[rustler::nif]
pub fn cookie_set<'a>(
    env: Env<'a>,
    handle: ResourceArc<CookieStoreHandle>,
    set_cookie: String,
    url: String,
) -> NifResult<Term<'a>> {
    handle
        .store
        .set(&set_cookie, &url)
        .map(|_| atoms::ok().encode(env))
        .map_err(|e| rustler::Error::Term(Box::new(ObscuraxError::from_obscura(&e))))
}

#[rustler::nif]
pub fn cookie_get_all<'a>(
    env: Env<'a>,
    handle: ResourceArc<CookieStoreHandle>,
) -> NifResult<Term<'a>> {
    let cookies = handle.store.get_all();
    let terms: Vec<Term> = cookies.iter().map(|c| cookie_to_term(env, c)).collect();
    Ok((atoms::ok(), terms).encode(env))
}

#[rustler::nif]
pub fn cookie_get_for_url<'a>(
    env: Env<'a>,
    handle: ResourceArc<CookieStoreHandle>,
    url: String,
) -> NifResult<Term<'a>> {
    let cookies = handle
        .store
        .get_for_url(&url)
        .map_err(|e| rustler::Error::Term(Box::new(ObscuraxError::from_obscura(&e))))?;
    let terms: Vec<Term> = cookies.iter().map(|c| cookie_to_term(env, c)).collect();
    Ok((atoms::ok(), terms).encode(env))
}

#[rustler::nif(schedule = "DirtyIo")]
pub fn cookie_save<'a>(
    env: Env<'a>,
    handle: ResourceArc<CookieStoreHandle>,
    path: String,
) -> NifResult<Term<'a>> {
    handle
        .store
        .save_to_file(Path::new(&path))
        .map(|_| atoms::ok().encode(env))
        .map_err(|e| rustler::Error::Term(Box::new(ObscuraxError::from_obscura(&e))))
}

#[rustler::nif(schedule = "DirtyIo")]
pub fn cookie_load<'a>(
    env: Env<'a>,
    handle: ResourceArc<CookieStoreHandle>,
    path: String,
) -> NifResult<Term<'a>> {
    let count = handle
        .store
        .load_from_file(Path::new(&path))
        .map_err(|e| rustler::Error::Term(Box::new(ObscuraxError::from_obscura(&e))))?;
    Ok((atoms::ok(), count).encode(env))
}
