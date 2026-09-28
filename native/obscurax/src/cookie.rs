use std::path::Path;
use std::sync::Arc;

use obscura_net::{CookieInfo, CookieJar};
use rustler::{Encoder, Env, NifResult, Resource, ResourceArc, Term};

use crate::atoms;
use crate::error::ObscuraxError;

pub struct CookieStoreHandle {
    pub jar: Arc<CookieJar>,
}

#[rustler::resource_impl]
impl Resource for CookieStoreHandle {}

impl CookieStoreHandle {
    pub fn new(jar: Arc<CookieJar>) -> Self {
        Self { jar }
    }
}

fn cookie_term<'a>(
    env: Env<'a>,
    name: &str,
    value: &str,
    domain: &str,
    path: &str,
    secure: bool,
    http_only: bool,
) -> Term<'a> {
    let pairs: Vec<(Term, Term)> = vec![
        (atoms::name().encode(env), name.encode(env)),
        (atoms::value().encode(env), value.encode(env)),
        (atoms::domain().encode(env), domain.encode(env)),
        (atoms::path().encode(env), path.encode(env)),
        (atoms::secure().encode(env), secure.encode(env)),
        (atoms::http_only().encode(env), http_only.encode(env)),
    ];
    rustler::Term::map_from_pairs(env, &pairs).unwrap_or(atoms::nil().encode(env))
}

fn cookie_info_term<'a>(env: Env<'a>, c: &CookieInfo) -> Term<'a> {
    cookie_term(
        env,
        &c.name,
        &c.value,
        &c.domain,
        &c.path,
        c.secure,
        c.http_only,
    )
}

fn parse_url(url: &str) -> NifResult<url::Url> {
    url::Url::parse(url).map_err(|e| {
        rustler::Error::Term(Box::new(ObscuraxError::internal(format!(
            "invalid url: {e}"
        ))))
    })
}

#[rustler::nif]
pub fn cookie_set<'a>(
    env: Env<'a>,
    handle: ResourceArc<CookieStoreHandle>,
    set_cookie: String,
    url: String,
) -> NifResult<Term<'a>> {
    handle.jar.set_cookie(&set_cookie, &parse_url(&url)?);
    Ok(atoms::ok().encode(env))
}

#[rustler::nif]
pub fn cookie_get_all<'a>(
    env: Env<'a>,
    handle: ResourceArc<CookieStoreHandle>,
) -> NifResult<Term<'a>> {
    let cookies: Vec<Term> = handle
        .jar
        .get_all_cookies()
        .iter()
        .map(|c| cookie_info_term(env, c))
        .collect();
    Ok((atoms::ok(), cookies).encode(env))
}

#[rustler::nif]
pub fn cookie_get_for_url<'a>(
    env: Env<'a>,
    handle: ResourceArc<CookieStoreHandle>,
    url: String,
) -> NifResult<Term<'a>> {
    let parsed = parse_url(&url)?;

    // Only the header carries cookies; the jar does not hand back the matched
    // CookieInfo values here, so recover name/value pairs from the join and
    // attribute them to the request host exactly as before.
    let cookies: Vec<Term> = handle
        .jar
        .get_cookie_header_same_site(&parsed)
        .split("; ")
        .filter(|s| !s.is_empty())
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let name = parts.next()?;
            let value = parts.next().unwrap_or("");
            let domain = parsed.host_str()?;
            Some(cookie_term(env, name, value, domain, "/", false, false))
        })
        .collect();
    Ok((atoms::ok(), cookies).encode(env))
}

#[rustler::nif(schedule = "DirtyIo")]
pub fn cookie_save<'a>(
    env: Env<'a>,
    handle: ResourceArc<CookieStoreHandle>,
    path: String,
) -> NifResult<Term<'a>> {
    handle.jar.save_to_file(Path::new(&path)).map_err(|e| {
        rustler::Error::Term(Box::new(ObscuraxError::internal(format!(
            "save cookies: {e}"
        ))))
    })?;
    Ok(atoms::ok().encode(env))
}

#[rustler::nif(schedule = "DirtyIo")]
pub fn cookie_load<'a>(
    env: Env<'a>,
    handle: ResourceArc<CookieStoreHandle>,
    path: String,
) -> NifResult<Term<'a>> {
    let count = handle.jar.load_from_file(Path::new(&path)).map_err(|e| {
        rustler::Error::Term(Box::new(ObscuraxError::internal(format!(
            "load cookies: {e}"
        ))))
    })?;
    Ok((atoms::ok(), count).encode(env))
}
