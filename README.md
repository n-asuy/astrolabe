<p align="center">
	<img src="image.jpg" alt="Astrolabe" />
	<h1 align="center"><b>Astrolabe</b></h1>
	<p align="center">
    SaaS starter kit built as a Turborepo monorepo.
  </p>
</p>

## What's included

### Core Technologies
[Vite](https://vite.dev/) - Build tool<br>
[React Router v7](https://reactrouter.com/) - SPA routing (SPA mode)<br>
[Turborepo](https://turbo.build) - Build system<br>
[Bun](https://bun.sh/) - Package manager & runtime<br>
[TypeScript](https://www.typescriptlang.org/) - Type safety<br>
[TailwindCSS](https://tailwindcss.com/) - Styling<br>
[Shadcn](https://ui.shadcn.com/) - UI components<br>

### Backend & Infrastructure
[Cloudflare Workers](https://workers.cloudflare.com/) - Edge computing (Rust)<br>
[Cloudflare D1](https://developers.cloudflare.com/d1/) - Edge-native SQLite database<br>
[Supabase Auth](https://supabase.com/auth) - Authentication<br>
[Stripe](https://stripe.com) - Billing<br>

### Development Tools
[Biome](https://biomejs.dev) - Linter & formatter<br>
[Sherif](https://github.com/QuiiBz/sherif) - Monorepo linting<br>

## Directory Structure

```
apps/
  app/            Vite SPA (React Router v7, Tailwind v3, shadcn)
  api/            Rust Cloudflare Worker (D1, Stripe)
packages/
  supabase/       Supabase client and database types
  ui/             Shared UI components (shadcn/Radix)
  logger/         Shared logging (Pino)
tooling/
  typescript/     Shared TypeScript configurations
```

## Prerequisites

- [Bun](https://bun.sh/) (v1.1.26 or later)
- [Rust](https://rustup.rs/) with `wasm32-unknown-unknown` target
- [Cloudflare account](https://cloudflare.com) (for Workers & D1)
- [Supabase account](https://supabase.com) (for authentication)
- [Stripe account](https://stripe.com) (for billing)

## Getting Started

1. Install dependencies:

```bash
bun i
```

2. Copy environment files:

```bash
cp apps/app/.env.example apps/app/.env
cp apps/api/.dev.vars.example apps/api/.dev.vars
```

3. Set up services:

   **Supabase (Authentication):**
   - Create a project at [supabase.com](https://supabase.com)
   - Copy the project URL and anon key to `apps/app/.env`
   - Copy the project URL to `apps/api/.dev.vars`

   **Cloudflare (API & Database):**
   - Install Wrangler CLI: `bun add -g wrangler`
   - Authenticate: `wrangler login`
   - Create D1 database: `wrangler d1 create astrolabe-db`
   - Update `apps/api/wrangler.toml` with your database ID

   **Stripe (Billing):**
   - Copy secret key and webhook secret to `apps/api/.dev.vars`

4. Set up the database:

```bash
cd apps/api
wrangler d1 migrations apply APP_DB --local
```

5. Start development:

```bash
bun dev          # Start all apps in parallel
bun dev:app      # SPA frontend only
bun dev:api      # Rust API only (port 5286)
```

### Environment Variables

| App | Variable | Description |
|-----|----------|-------------|
| app | `VITE_SUPABASE_URL` | Supabase project URL |
| app | `VITE_SUPABASE_ANON_KEY` | Supabase anonymous key |
| app | `VITE_API_URL` | API endpoint (default: `http://localhost:5286`) |
| api | `SUPABASE_URL` | Supabase project URL |
| api | `STRIPE_SECRET_KEY` | Stripe secret key |
| api | `STRIPE_WEBHOOK_SECRET` | Stripe webhook signing secret |
| api | `APP_BASE_URL` | Frontend URL for Stripe redirects |

## API Development

The API is a Rust Cloudflare Worker with D1 SQLite:

```bash
cd apps/api

# Type check
cargo check --target wasm32-unknown-unknown

# Format
cargo fmt

# Local database migrations
wrangler d1 migrations apply APP_DB --local

# Deploy
wrangler deploy

# Production database migrations
wrangler d1 migrations apply APP_DB --remote
```

### API Routes

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/health` | No | Health check |
| GET | `/api/session` | Yes | Current user session |
| GET | `/api/stripe/products` | No | List Stripe products |
| GET | `/api/stripe/prices` | No | List Stripe prices |
| POST | `/api/stripe/checkout/sessions` | Yes | Create checkout session |
| POST | `/api/stripe/portal/sessions` | Yes | Create billing portal session |
| POST | `/api/webhooks/stripe` | No | Stripe webhook receiver |

## Common Commands

```bash
bun dev          # Start all apps
bun build        # Build all apps
bun typecheck    # TypeScript checking
bun lint         # Lint with Biome + Sherif
bun format       # Format with Biome
bun clean        # Clean build artifacts
```

## Deployment

### SPA (Cloudflare Pages / any static host)

```bash
bun run --filter @astrolabe/app build
# Deploy build/client/ to your static host
```

### API (Cloudflare Workers)

```bash
cd apps/api
wrangler deploy
wrangler d1 migrations apply APP_DB --remote
```
