# CLAUDE.md

This file provides guidance to Claude Code when working with code in this repository.

## Project Overview

A SaaS starter kit built as a Turborepo monorepo with a Vite + React Router v7 SPA frontend and a Rust Cloudflare Workers API backend.

## Common Development Commands

### Development
```bash
bun i                    # Install dependencies
bun dev                  # Start all apps in parallel
bun dev:app              # SPA frontend (Vite dev server)
bun dev:api              # Rust API on Cloudflare Workers (port 5286)
```

### Code Quality
```bash
bun lint                 # Lint entire codebase
bun format               # Format code with Biome
bun typecheck            # Type checking
bun lint:repo:fix        # Fix Sherif monorepo linting issues
```

### API (Rust)
```bash
cd apps/api
cargo fmt                # Format Rust code
cargo check --target wasm32-unknown-unknown  # Type check
wrangler deploy          # Deploy to Cloudflare Workers
wrangler d1 migrations apply APP_DB --local   # Apply D1 migrations locally
wrangler d1 migrations apply APP_DB --remote  # Apply D1 migrations to production
```

### Build & Clean
```bash
bun build                # Build all apps
bun clean                # Clean all build artifacts
bun clean:workspaces     # Clean workspace artifacts
```

## Architecture

### Monorepo Structure
- **apps/app**: SPA frontend (Vite + React Router v7, SPA mode, Tailwind v3 + shadcn)
- **apps/api**: REST API (Rust Cloudflare Worker, D1 SQLite, Stripe)
- **packages/supabase**: Supabase client and database types
- **packages/ui**: Shared UI components (shadcn/Radix, Tailwind)
- **packages/logger**: Shared logging (Pino)
- **tooling/typescript**: Shared TypeScript configurations

### Authentication
- **Supabase Auth** handles user authentication
- SPA uses `@supabase/supabase-js` browser client
- API verifies Supabase JWT tokens via JWKS endpoint
- Auth context provided via React Context (`app/lib/auth.tsx`)

### API Communication
- SPA calls Rust API at `VITE_API_URL` (default: `http://localhost:5286`)
- Auth: Bearer token from Supabase session passed in Authorization header
- API routes: `/api/health`, `/api/session`, `/api/stripe/*`, `/api/webhooks/stripe`

### Stripe Integration
- Checkout sessions, billing portal, products/prices listing via API
- Webhook verification with HMAC-SHA256
- Subscription management stored in D1 database

### Environment Variables
- **apps/app**: `VITE_SUPABASE_URL`, `VITE_SUPABASE_ANON_KEY`, `VITE_API_URL`
- **apps/api**: `SUPABASE_URL`, `STRIPE_SECRET_KEY`, `STRIPE_WEBHOOK_SECRET`, `APP_BASE_URL`

## Code Style
- Functional TypeScript, no classes
- Biome for linting/formatting
- shadcn UI components from `@astrolabe/ui`
- Tailwind CSS v3 with CSS variable theming
