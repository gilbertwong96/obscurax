rustler::atoms! {
    world,
    ok,
    error,
    true_ = "true",
    false_ = "false",
    nil,
    obscurax_result,
    obscurax_request,
    obscurax_response,
    obscurax_intercept,
    continue_ = "continue",
    fulfill,
    fail,
    navigation,
    js_eval,
    timeout,
    element_not_found,
    no_page,
    page_closed,
    internal,
    url,
    method,
    resource_type,
    status,
    name,
    value,
    domain,
    path,
    secure,
    http_only,
}

use rustler::{Env, Encoder, Term};
use serde_json::Value;

pub fn json_to_term<'a>(env: Env<'a>, v: &Value) -> Term<'a> {
    match v {
        Value::Null => nil().encode(env),
        Value::Bool(b) => {
            if *b {
                true_().encode(env)
            } else {
                false_().encode(env)
            }
        }
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.encode(env)
            } else if let Some(f) = n.as_f64() {
                if f.fract() == 0.0 && f.is_finite() {
                    (f as i64).encode(env)
                } else {
                    f.encode(env)
                }
            } else {
                n.to_string().encode(env)
            }
        }
        Value::String(s) => s.encode(env),
        Value::Array(arr) => {
            let terms: Vec<Term> = arr.iter().map(|v| json_to_term(env, v)).collect();
            terms.encode(env)
        }
        Value::Object(obj) => {
            let pairs: Vec<(Term, Term)> = obj
                .iter()
                .map(|(k, v)| (k.encode(env), json_to_term(env, v)))
                .collect();
            rustler::Term::map_from_pairs(env, &pairs).unwrap_or(nil().encode(env))
        }
    }
}
