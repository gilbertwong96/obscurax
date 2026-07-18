use rustler::{Atom, NifException, NifMap};

use crate::atoms;

/// Error context carried as an Elixir map. Built with atom keys so Elixir
/// consumers can pattern-match on `ctx.selector`, `ctx.url`, etc.
#[derive(NifMap)]
pub struct ErrorContext {
    pub url: Option<String>,
    pub selector: Option<String>,
    pub timeout_ms: Option<u64>,
    pub node_id: Option<u64>,
    pub expression: Option<String>,
}

impl ErrorContext {
    fn empty() -> Self {
        Self {
            url: None,
            selector: None,
            timeout_ms: None,
            node_id: None,
            expression: None,
        }
    }
}

#[derive(NifException)]
#[module = "Obscurax.Error"]
pub struct ObscuraxError {
    pub kind: Atom,
    pub message: String,
    pub context: ErrorContext,
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
            context: ErrorContext::empty(),
        }
    }

    pub fn page_closed() -> Self {
        ObscuraxError {
            kind: kind_atom("page_closed"),
            message: "page thread has exited".to_string(),
            context: ErrorContext::empty(),
        }
    }

    #[allow(dead_code)]
    pub fn navigation(url: &str, message: impl Into<String>) -> Self {
        ObscuraxError {
            kind: kind_atom("navigation"),
            message: message.into(),
            context: ErrorContext {
                url: Some(url.to_string()),
                ..ErrorContext::empty()
            },
        }
    }

    pub fn timeout(
        selector: Option<&str>,
        timeout_ms: Option<u64>,
        message: impl Into<String>,
    ) -> Self {
        ObscuraxError {
            kind: kind_atom("timeout"),
            message: message.into(),
            context: ErrorContext {
                selector: selector.map(str::to_string),
                timeout_ms,
                ..ErrorContext::empty()
            },
        }
    }

    pub fn element_not_found(
        selector: Option<&str>,
        node_id: Option<u64>,
        message: impl Into<String>,
    ) -> Self {
        ObscuraxError {
            kind: kind_atom("element_not_found"),
            message: message.into(),
            context: ErrorContext {
                selector: selector.map(str::to_string),
                node_id,
                ..ErrorContext::empty()
            },
        }
    }

    #[allow(dead_code)]
    pub fn js_eval(expression: &str, message: impl Into<String>) -> Self {
        ObscuraxError {
            kind: kind_atom("js_eval"),
            message: message.into(),
            context: ErrorContext {
                expression: Some(expression.to_string()),
                ..ErrorContext::empty()
            },
        }
    }
}

pub fn nif_error(kind: &str, message: impl Into<String>) -> ObscuraxError {
    ObscuraxError {
        kind: kind_atom(kind),
        message: message.into(),
        context: ErrorContext::empty(),
    }
}
