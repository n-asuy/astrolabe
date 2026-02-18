import { Button } from "@astrolabe/ui/button";
import { useEffect, useState } from "react";
import { getSupabase } from "~/lib/supabase";

type Product = {
  id: string;
  name: string;
  description?: string | null;
  defaultPriceId?: string | null;
};

type Price = {
  id: string;
  productId: string;
  unitAmount: number;
  currency: string;
  interval?: string;
  trialPeriodDays?: number;
};

export default function PricingPage() {
  const [products, setProducts] = useState<Product[]>([]);
  const [prices, setPrices] = useState<Price[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const apiUrl = import.meta.env.VITE_API_URL || "http://localhost:5286";
    Promise.all([
      fetch(`${apiUrl}/api/stripe/products`).then((r) => r.ok ? r.json() : []),
      fetch(`${apiUrl}/api/stripe/prices`).then((r) => r.ok ? r.json() : []),
    ]).then(([prods, prs]) => {
      setProducts(prods);
      setPrices(prs);
      setLoading(false);
    });
  }, []);

  if (loading) {
    return (
      <div className="flex min-h-screen items-center justify-center">
        <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
      </div>
    );
  }

  const basePlan = products.find((p) => p.name === "Base");
  const plusPlan = products.find((p) => p.name === "Plus");
  const basePrice = prices.find((pr) => pr.productId === basePlan?.id);
  const plusPrice = prices.find((pr) => pr.productId === plusPlan?.id);

  return (
    <main className="mx-auto grid max-w-4xl gap-8 px-4 py-12 md:grid-cols-2">
      <PricingCard
        name={basePlan?.name || "Base"}
        price={basePrice?.unitAmount || 800}
        interval={basePrice?.interval || "month"}
        trialDays={basePrice?.trialPeriodDays || 7}
        features={[
          "Unlimited Usage",
          "Unlimited Workspace Members",
          "Email Support",
        ]}
        priceId={basePrice?.id}
      />
      <PricingCard
        name={plusPlan?.name || "Plus"}
        price={plusPrice?.unitAmount || 1200}
        interval={plusPrice?.interval || "month"}
        trialDays={plusPrice?.trialPeriodDays || 7}
        features={[
          "Everything in Base, and:",
          "Early Access to New Features",
          "24/7 Support + Slack Access",
        ]}
        priceId={plusPrice?.id}
      />
    </main>
  );
}

function PricingCard({
  name,
  price,
  interval,
  trialDays,
  features,
  priceId,
}: {
  name: string;
  price: number;
  interval: string;
  trialDays: number;
  features: string[];
  priceId?: string;
}) {
  const [submitting, setSubmitting] = useState(false);

  const handleCheckout = async () => {
    if (!priceId) return;
    setSubmitting(true);

    const apiUrl = import.meta.env.VITE_API_URL || "http://localhost:5286";
    const { data } = await getSupabase().auth.getSession();
    const token = data.session?.access_token;

    const res = await fetch(`${apiUrl}/api/stripe/checkout/sessions`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
      },
      body: JSON.stringify({ priceId }),
    });

    if (res.ok) {
      const { url } = await res.json();
      if (url) window.location.href = url;
    }
    setSubmitting(false);
  };

  return (
    <div className="rounded-lg border p-6">
      <h2 className="mb-2 text-2xl font-semibold">{name}</h2>
      <p className="mb-4 text-sm text-muted-foreground">
        with {trialDays} day free trial
      </p>
      <p className="mb-6 text-4xl font-medium">
        ${price / 100}{" "}
        <span className="text-base font-normal text-muted-foreground">
          / {interval}
        </span>
      </p>
      <ul className="mb-6 space-y-2">
        {features.map((f) => (
          <li key={f} className="text-sm text-muted-foreground">
            {f}
          </li>
        ))}
      </ul>
      <Button
        onClick={handleCheckout}
        disabled={!priceId || submitting}
        className="w-full"
      >
        {submitting ? "Loading..." : priceId ? "Subscribe" : "Unavailable"}
      </Button>
    </div>
  );
}
