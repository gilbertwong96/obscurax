use rustler::{Atom, NifException, SerdeTerm};

use crate::atoms;

#[derive(NifException)]
#[module = "Obscurax.Error"]
pub struct ObscuraxError {
    pub kind: Atom,
    pub message: String,
    pub context: SerdeTerm<serde_json::Value>,
}

fn kind_atom(kind: &str) -> Atom {
    match kind {
        "navigation" => atoms::navigation(),
        "js_eval" => atoms::js_eval(),
        "timeout" => atoms::timeout(),
        "element_not_found" => atoms::element_not_found(),
        "no_page" => atoms::no_page(),
        "page_closed" => atoms::page_closed(),
        // "internal" and any unknown kind fall back to internal
        _ => atoms::internal(),
    }
}

impl ObscuraxError {
    pub fn from_obscura(e: &obscura::Error) -> Self {
        let (kind, message) = match e {
            obscura::Error::Navigation(s) => ("navigation", s.clone()),
            obscura::Error::JsEval(s) => ("js_eval", s.clone()),
            obscura::Error::Timeout(s) => ("timeout", s.clone()),
            obscura::Error::ElementNotFound(s) => ("element_not_found", s.clone()),
            obscura::Error::NoPage => ("no_page", "no page session".to_string()),
            obscura::Error::Internal(e) => ("internal", e.to_string()),
        };
        ObscuraxError {
            kind: kind_atom(kind),
            message,
            context: SerdeTerm(serde_json::Value::Null),
        }
    }

    pub fn page_closed() -> Self {
        ObscuraxError {
            kind: kind_atom("page_closed"),
            message: "page thread has exited".to_string(),
            context: SerdeTerm(serde_json::Value::Null),
        }
    }
}

pub fn nif_error(kind: &str, message: impl Into<String>) -> ObscuraxError {
    ObscuraxError {
        kind: kind_atom(kind),
        message: message.into(),
        context: SerdeTerm(serde_json::Value::Null),
    }
}
