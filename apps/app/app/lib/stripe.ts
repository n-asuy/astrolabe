// Stripe REST client - fetch-based, runtime agnostic.
// Ported from the original Hono API.

type StripeCheckoutSession = {
  id: string;
  url?: string;
  customer?: string | { id: string } | null;
  subscription?: string | { id: string } | null;
  client_reference_id?: string | null;
};

type StripePrice = {
  id: string;
  product: string | { id: string; name?: string };
};

type StripeSubscription = {
  id: string;
  status: string;
  customer: string;
  items: {
    data: Array<{
      price: StripePrice;
      plan?: { product?: string; name?: string };
    }>;
  };
};

export async function stripeGet<T>(
  secretKey: string,
  path: string,
  params?: Record<string, string | string[]>,
): Promise<T> {
  const url = new URL(`https://api.stripe.com/v1/${path}`);
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      if (Array.isArray(v)) {
        for (const val of v) url.searchParams.append(k, val);
      } else if (v != null) {
        url.searchParams.set(k, v);
      }
    }
  }
  const res = await fetch(url.toString(), {
    method: "GET",
    headers: {
      Authorization: `Bearer ${secretKey}`,
    },
  });
  if (!res.ok) {
    const txt = await res.text();
    throw new Error(`Stripe GET ${path} failed: ${res.status} ${txt}`);
  }
  return (await res.json()) as T;
}

export async function stripePostForm<T>(
  secretKey: string,
  path: string,
  body: Record<string, string | string[]>,
): Promise<T> {
  const form = new URLSearchParams();
  for (const [k, v] of Object.entries(body)) {
    if (Array.isArray(v)) {
      for (const val of v) form.append(k, val);
    } else if (v != null) {
      form.append(k, v);
    }
  }
  const res = await fetch(`https://api.stripe.com/v1/${path}`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${secretKey}`,
      "Content-Type": "application/x-www-form-urlencoded",
    },
    body: form.toString(),
  });
  if (!res.ok) {
    const txt = await res.text();
    throw new Error(`Stripe POST ${path} failed: ${res.status} ${txt}`);
  }
  return (await res.json()) as T;
}

export async function retrieveCheckoutSession(
  secretKey: string,
  sessionId: string,
): Promise<StripeCheckoutSession> {
  return stripeGet<StripeCheckoutSession>(
    secretKey,
    `checkout/sessions/${sessionId}`,
    { "expand[]": ["customer", "subscription"] },
  );
}

export async function retrieveSubscription(
  secretKey: string,
  subscriptionId: string,
): Promise<StripeSubscription> {
  return stripeGet<StripeSubscription>(
    secretKey,
    `subscriptions/${subscriptionId}`,
    { "expand[]": ["items.data.price.product"] },
  );
}

export async function verifyStripeSignature(
  payload: string,
  signatureHeader: string | null,
  secret: string,
  toleranceSeconds = 300,
): Promise<boolean> {
  try {
    const parsed = parseStripeSignature(signatureHeader);
    if (!parsed) return false;
    const { t, v1 } = parsed;
    const signedPayload = `${t}.${payload}`;
    const expected = await computeHmacSha256Hex(secret, signedPayload);
    const now = Math.floor(Date.now() / 1000);
    if (Math.abs(now - Number(t)) > toleranceSeconds) return false;
    return v1.some((sig) => timingSafeEqual(expected, sig));
  } catch {
    return false;
  }
}

function parseStripeSignature(header: string | null) {
  if (!header) return null;
  const parts = header.split(",").map((p) => p.trim());
  let t: string | null = null;
  const v1: string[] = [];
  for (const p of parts) {
    const [k, v] = p.split("=");
    if (k === "t") t = v;
    if (k === "v1" && v) v1.push(v);
  }
  if (!t || v1.length === 0) return null;
  return { t, v1 };
}

async function computeHmacSha256Hex(
  secret: string,
  message: string,
): Promise<string> {
  const enc = new TextEncoder();
  const key = await crypto.subtle.importKey(
    "raw",
    enc.encode(secret),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign"],
  );
  const sig = await crypto.subtle.sign("HMAC", key, enc.encode(message));
  return toHex(sig);
}

function toHex(buf: ArrayBuffer): string {
  const b = new Uint8Array(buf);
  let s = "";
  for (let i = 0; i < b.length; i++) s += b[i].toString(16).padStart(2, "0");
  return s;
}

function timingSafeEqual(a: string, b: string): boolean {
  if (a.length !== b.length) return false;
  let out = 0;
  for (let i = 0; i < a.length; i++) out |= a.charCodeAt(i) ^ b.charCodeAt(i);
  return out === 0;
}

export type { StripeSubscription, StripeCheckoutSession };
