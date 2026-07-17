use rustler::{NifException, SerdeTerm};

#[derive(NifException)]
#[module = "Obscurax.Error"]
pub struct ObscuraxError {
    pub kind: String,
    pub message: String,
    pub context: SerdeTerm<serde_json::Value>,
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
            kind: kind.to_string(),
            message,
            context: SerdeTerm(serde_json::Value::Null),
        }
    }

    pub fn page_closed() -> Self {
        ObscuraxError {
            kind: "page_closed".to_string(),
            message: "page thread has exited".to_string(),
            context: SerdeTerm(serde_json::Value::Null),
        }
    }
}

pub fn nif_error(kind: &str, message: impl Into<String>) -> ObscuraxError {
    ObscuraxError {
        kind: kind.to_string(),
        message: message.into(),
        context: SerdeTerm(serde_json::Value::Null),
    }
}
