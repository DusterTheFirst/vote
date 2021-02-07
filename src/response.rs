use std::collections::HashMap;

use oauth2::http::StatusCode;
use wasm_bindgen::JsValue;
use web_sys::ResponseInit;

pub enum Response {
    // Template(Box<dyn Template>),
    Content(String),
    StatusCode(StatusCode),
    Redirect(String),
}

impl From<Response> for web_sys::Response {
    fn from(response: Response) -> Self {
        let (content, status_code, headers) = match response {
            Response::Content(content) => (
                Some(content),
                StatusCode::OK,
                [("Content-Type", "text/html;charset=UTF-8")]
                    .iter()
                    .map(|x| (x.0.to_string(), x.1.to_string()))
                    .collect::<HashMap<String, String>>(),
            ),
            Response::StatusCode(code) => (
                Some(format!(
                    "{}: {}",
                    code.as_str(),
                    code.canonical_reason().unwrap_or("Unknown Status")
                )),
                code,
                HashMap::new(),
            ),
            Response::Redirect(to) => (
                None,
                StatusCode::TEMPORARY_REDIRECT,
                [("Location".to_string(), to)].iter().cloned().collect(),
            ),
        };

        web_sys::Response::new_with_opt_str_and_init(
            content.as_ref().map(String::as_str),
            ResponseInit::new()
                .status(status_code.as_u16())
                .headers(&JsValue::from_serde(&headers).unwrap()),
        )
        .unwrap()
    }
}

impl From<StatusCode> for Response {
    fn from(code: StatusCode) -> Self {
        Self::StatusCode(code)
    }
}
