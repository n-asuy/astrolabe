use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use worker::Env;

use crate::error::ApiError;

#[derive(Deserialize)]
pub struct StripeList<T> {
    pub data: Vec<T>,
}

#[derive(Deserialize)]
pub struct StripeProduct {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub default_price: Option<StripePriceOrId>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum StripePriceOrId {
    Expanded(Box<StripePrice>),
    Id(String),
}

#[derive(Deserialize)]
pub struct StripePrice {
    pub id: String,
    pub product: Option<StripeProductOrId>,
    pub unit_amount: Option<i64>,
    pub currency: Option<String>,
    pub recurring: Option<StripeRecurring>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum StripeProductOrId {
    Expanded(Box<StripeProduct>),
    Id(String),
}

#[derive(Deserialize)]
pub struct StripeRecurring {
    pub interval: Option<String>,
    pub trial_period_days: Option<i64>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct StripeCheckoutSession {
    pub id: String,
    pub url: Option<String>,
    pub customer: Option<String>,
    pub subscription: Option<String>,
    pub client_reference_id: Option<String>,
}

fn get_secret(env: &Env, name: &str) -> Result<String, ApiError> {
    env.secret(name)
        .or_else(|_| env.var(name))
        .map(|v| v.to_string())
        .map_err(|_| ApiError::Internal(format!("Missing secret: {}", name)))
}

pub fn stripe_secret_key(env: &Env) -> Result<String, ApiError> {
    get_secret(env, "STRIPE_SECRET_KEY")
}

pub fn stripe_webhook_secret(env: &Env) -> Result<String, ApiError> {
    get_secret(env, "STRIPE_WEBHOOK_SECRET")
}

pub fn app_base_url(env: &Env) -> String {
    env.var("APP_BASE_URL")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "http://localhost:5285".to_string())
}

pub async fn stripe_get<T: serde::de::DeserializeOwned>(
    secret_key: &str,
    path: &str,
    params: &[(&str, &str)],
) -> Result<T, ApiError> {
    let mut url = worker::Url::parse(&format!("https://api.stripe.com/v1/{}", path))
        .map_err(|_| ApiError::Internal("Invalid Stripe URL".to_string()))?;

    for (k, v) in params {
        url.query_pairs_mut().append_pair(k, v);
    }

    let headers = worker::Headers::new();
    headers
        .set("Authorization", &format!("Bearer {}", secret_key))
        .ok();

    let req = worker::Request::new_with_init(
        url.as_str(),
        worker::RequestInit::new()
            .with_method(worker::Method::Get)
            .with_headers(headers),
    )
    .map_err(|e| ApiError::Internal(format!("Request build failed: {}", e)))?;

    let mut resp = worker::Fetch::Request(req)
        .send()
        .await
        .map_err(|e| ApiError::Internal(format!("Stripe GET failed: {}", e)))?;

    if resp.status_code() >= 400 {
        let txt = resp.text().await.unwrap_or_default();
        return Err(ApiError::Internal(format!(
            "Stripe GET {} failed: {} {}",
            path,
            resp.status_code(),
            txt
        )));
    }

    resp.json()
        .await
        .map_err(|e| ApiError::Internal(format!("Stripe response parse failed: {}", e)))
}

pub async fn stripe_post_form<T: serde::de::DeserializeOwned>(
    secret_key: &str,
    path: &str,
    body: &[(&str, &str)],
) -> Result<T, ApiError> {
    let form: String = body
        .iter()
        .map(|(k, v)| format!("{}={}", url_encode(k), url_encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    let url = format!("https://api.stripe.com/v1/{}", path);

    let headers = worker::Headers::new();
    headers
        .set("Authorization", &format!("Bearer {}", secret_key))
        .ok();
    headers
        .set("Content-Type", "application/x-www-form-urlencoded")
        .ok();

    let req = worker::Request::new_with_init(
        &url,
        worker::RequestInit::new()
            .with_method(worker::Method::Post)
            .with_headers(headers)
            .with_body(Some(wasm_bindgen::JsValue::from_str(&form))),
    )
    .map_err(|e| ApiError::Internal(format!("Request build failed: {}", e)))?;

    let mut resp = worker::Fetch::Request(req)
        .send()
        .await
        .map_err(|e| ApiError::Internal(format!("Stripe POST failed: {}", e)))?;

    if resp.status_code() >= 400 {
        let txt = resp.text().await.unwrap_or_default();
        return Err(ApiError::Internal(format!(
            "Stripe POST {} failed: {} {}",
            path,
            resp.status_code(),
            txt
        )));
    }

    resp.json()
        .await
        .map_err(|e| ApiError::Internal(format!("Stripe response parse failed: {}", e)))
}

fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => {
                out.push('%');
                out.push_str(&format!("{:02X}", b));
            }
        }
    }
    out
}

pub fn verify_webhook_signature(
    payload: &str,
    sig_header: &str,
    secret: &str,
) -> Result<bool, ApiError> {
    let mut timestamp: Option<&str> = None;
    let mut signatures: Vec<&str> = Vec::new();

    for part in sig_header.split(',') {
        let part = part.trim();
        if let Some(t) = part.strip_prefix("t=") {
            timestamp = Some(t);
        } else if let Some(v) = part.strip_prefix("v1=") {
            signatures.push(v);
        }
    }

    let t = timestamp.ok_or_else(|| ApiError::BadRequest("Missing timestamp".to_string()))?;
    if signatures.is_empty() {
        return Err(ApiError::BadRequest("Missing signature".to_string()));
    }

    let signed_payload = format!("{}.{}", t, payload);
    let expected = hmac_sha256_hex(secret, &signed_payload);

    let now = (js_sys::Date::now() / 1000.0) as i64;
    let ts: i64 = t
        .parse()
        .map_err(|_| ApiError::BadRequest("Invalid timestamp".to_string()))?;
    if (now - ts).abs() > 300 {
        return Ok(false);
    }

    Ok(signatures.iter().any(|s| timing_safe_eq(&expected, s)))
}

fn hmac_sha256_hex(secret: &str, message: &str) -> String {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(message.as_bytes());
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

fn timing_safe_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        diff |= x ^ y;
    }
    diff == 0
}
