use serde::Deserialize;
use worker::*;

mod auth;
mod error;
mod stripe;

use auth::require_user;
use error::ApiError;
use stripe::{
    app_base_url, stripe_get, stripe_post_form, stripe_secret_key, stripe_webhook_secret,
    verify_webhook_signature, StripePriceOrId, StripeProductOrId,
};

fn add_cors_headers(resp: &mut Response) -> Result<()> {
    let headers = resp.headers_mut();
    headers.set("Access-Control-Allow-Origin", "*")?;
    headers.set(
        "Access-Control-Allow-Methods",
        "GET, POST, PUT, DELETE, OPTIONS",
    )?;
    headers.set(
        "Access-Control-Allow-Headers",
        "Content-Type, Authorization",
    )?;
    headers.set("Access-Control-Max-Age", "86400")?;
    Ok(())
}

fn cors_preflight() -> Result<Response> {
    let mut resp = Response::empty()?.with_status(204);
    add_cors_headers(&mut resp)?;
    Ok(resp)
}

fn json_response<T: serde::Serialize>(data: &T) -> Result<Response> {
    let mut resp = Response::from_json(data)?;
    add_cors_headers(&mut resp)?;
    Ok(resp)
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    if req.method() == Method::Options {
        return cors_preflight();
    }

    let router = Router::new();

    let result = router
        .get_async("/api/health", |_, _| async {
            json_response(&serde_json::json!({ "status": "ok" }))
        })
        .get_async("/api/session", handle_session)
        .get_async("/api/stripe/products", handle_list_products)
        .get_async("/api/stripe/prices", handle_list_prices)
        .post_async("/api/stripe/checkout/sessions", handle_create_checkout)
        .post_async("/api/stripe/billing-portal/sessions", handle_create_portal)
        .post_async("/api/webhooks/stripe", handle_stripe_webhook)
        .run(req, env)
        .await;

    match result {
        Ok(mut resp) => {
            add_cors_headers(&mut resp)?;
            Ok(resp)
        }
        Err(e) => {
            let mut resp = Response::error(format!("{}", e), 500)?;
            add_cors_headers(&mut resp)?;
            Ok(resp)
        }
    }
}

async fn handle_session(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let user = match require_user(&req, &ctx.env).await {
        Ok(u) => u,
        Err(e) => return e.into_response(),
    };

    json_response(&serde_json::json!({
        "userId": user.user_id,
    }))
}

async fn handle_list_products(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let secret = match stripe_secret_key(&ctx.env) {
        Ok(k) => k,
        Err(e) => return e.into_response(),
    };

    let list: stripe::StripeList<stripe::StripeProduct> = match stripe_get(
        &secret,
        "products",
        &[
            ("active", "true"),
            ("expand[]", "data.default_price"),
            ("limit", "100"),
        ],
    )
    .await
    {
        Ok(l) => l,
        Err(e) => return e.into_response(),
    };

    let products: Vec<serde_json::Value> = list
        .data
        .iter()
        .map(|p| {
            let default_price_id = match &p.default_price {
                Some(StripePriceOrId::Expanded(price)) => Some(price.id.clone()),
                Some(StripePriceOrId::Id(id)) => Some(id.clone()),
                None => None,
            };
            serde_json::json!({
                "id": p.id,
                "name": p.name,
                "description": p.description,
                "defaultPriceId": default_price_id,
            })
        })
        .collect();

    json_response(&products)
}

async fn handle_list_prices(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let secret = match stripe_secret_key(&ctx.env) {
        Ok(k) => k,
        Err(e) => return e.into_response(),
    };

    let list: stripe::StripeList<stripe::StripePrice> = match stripe_get(
        &secret,
        "prices",
        &[
            ("active", "true"),
            ("type", "recurring"),
            ("expand[]", "data.product"),
            ("limit", "100"),
        ],
    )
    .await
    {
        Ok(l) => l,
        Err(e) => return e.into_response(),
    };

    let prices: Vec<serde_json::Value> = list
        .data
        .iter()
        .map(|p| {
            let product_id = match &p.product {
                Some(StripeProductOrId::Expanded(prod)) => prod.id.clone(),
                Some(StripeProductOrId::Id(id)) => id.clone(),
                None => String::new(),
            };
            serde_json::json!({
                "id": p.id,
                "productId": product_id,
                "unitAmount": p.unit_amount.unwrap_or(0),
                "currency": p.currency.as_deref().unwrap_or("usd"),
                "interval": p.recurring.as_ref().and_then(|r| r.interval.as_deref()),
                "trialPeriodDays": p.recurring.as_ref().and_then(|r| r.trial_period_days),
            })
        })
        .collect();

    json_response(&prices)
}

#[derive(Deserialize)]
struct CheckoutRequest {
    #[serde(rename = "priceId")]
    price_id: String,
}

async fn handle_create_checkout(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let user = match require_user(&req, &ctx.env).await {
        Ok(u) => u,
        Err(e) => return e.into_response(),
    };

    let body: CheckoutRequest = match req.json().await {
        Ok(b) => b,
        Err(_) => return ApiError::BadRequest("Invalid request body".to_string()).into_response(),
    };

    let secret = match stripe_secret_key(&ctx.env) {
        Ok(k) => k,
        Err(e) => return e.into_response(),
    };

    let base_url = app_base_url(&ctx.env);
    let success_url = format!(
        "{}/pricing?checkout=success&session_id={{CHECKOUT_SESSION_ID}}",
        base_url
    );
    let cancel_url = format!("{}/pricing", base_url);

    let session: stripe::StripeCheckoutSession = match stripe_post_form(
        &secret,
        "checkout/sessions",
        &[
            ("payment_method_types[]", "card"),
            ("line_items[0][price]", &body.price_id),
            ("line_items[0][quantity]", "1"),
            ("mode", "subscription"),
            ("success_url", &success_url),
            ("cancel_url", &cancel_url),
            ("client_reference_id", &user.user_id),
            ("allow_promotion_codes", "true"),
            ("subscription_data[trial_period_days]", "14"),
        ],
    )
    .await
    {
        Ok(s) => s,
        Err(e) => return e.into_response(),
    };

    json_response(&serde_json::json!({ "url": session.url }))
}

async fn handle_create_portal(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let _user = match require_user(&req, &ctx.env).await {
        Ok(u) => u,
        Err(e) => return e.into_response(),
    };

    // TODO: Look up stripe_customer_id from user's profile in database,
    // then create a billing portal session.
    ApiError::Internal("Billing portal not yet implemented".to_string()).into_response()
}

#[derive(Deserialize)]
struct StripeEvent {
    #[serde(rename = "type")]
    event_type: String,
    #[allow(dead_code)]
    data: StripeEventData,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct StripeEventData {
    object: serde_json::Value,
}

async fn handle_stripe_webhook(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let secret = match stripe_webhook_secret(&ctx.env) {
        Ok(s) => s,
        Err(e) => return e.into_response(),
    };

    let sig = req
        .headers()
        .get("stripe-signature")
        .ok()
        .flatten()
        .unwrap_or_default();

    let payload = match req.text().await {
        Ok(t) => t,
        Err(_) => return ApiError::BadRequest("Failed to read body".to_string()).into_response(),
    };

    match verify_webhook_signature(&payload, &sig, &secret) {
        Ok(true) => {}
        Ok(false) => return ApiError::BadRequest("Invalid signature".to_string()).into_response(),
        Err(e) => return e.into_response(),
    }

    let event: StripeEvent = match serde_json::from_str(&payload) {
        Ok(e) => e,
        Err(_) => return ApiError::BadRequest("Invalid event payload".to_string()).into_response(),
    };

    match event.event_type.as_str() {
        "customer.subscription.updated" | "customer.subscription.deleted" => {
            // TODO: Update subscription status in database
            // Extract customer, subscription_id, status, product from event.data.object
            worker::console_log!("Stripe webhook: {} processed", event.event_type);
        }
        _ => {
            worker::console_log!("Stripe webhook: unhandled event type {}", event.event_type);
        }
    }

    json_response(&serde_json::json!({ "received": true }))
}
