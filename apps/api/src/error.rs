use worker::Response;

pub enum ApiError {
    Unauthorized(String),
    BadRequest(String),
    Internal(String),
}

impl ApiError {
    pub fn into_response(self) -> worker::Result<Response> {
        let (status, message) = match self {
            ApiError::Unauthorized(m) => (401, m),
            ApiError::BadRequest(m) => (400, m),
            ApiError::Internal(m) => (500, m),
        };
        let body = serde_json::json!({ "error": message });
        let mut resp = Response::from_json(&body)?;
        let headers = resp.headers_mut();
        headers.set("Access-Control-Allow-Origin", "*")?;
        headers.set(
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization",
        )?;
        Ok(resp.with_status(status))
    }
}
