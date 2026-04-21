# TheGamblingFinance

A self-hosted web app for tracking the finances of your home poker nights:
buy-ins from the bank, IOU chip trades between players, and an end-of-night
settlement that tells everyone exactly who pays whom. Ships with a lifetime
cross-night leaderboard.

- **Backend** — Rust + Axum + sqlx + PostgreSQL
- **Frontend** — Vue 3 + Vite + Pinia + Tailwind CSS
- **Deploy** — Docker Compose on a VPS

The full design lives in [`plan.md`](./plan.md). This README is the quickstart.

## Quickstart (local)

```bash
cp .env.example .env             # adjust secrets + ADMIN_* if you want
docker compose -f docker/compose.yaml up --build
```

Services:

| Service | Port | URL |
|---|---|---|
| `web` (frontend) | 5173 | http://localhost:5173 |
| `api` (backend) | 8080 | http://localhost:8080/api/health |
| `db` (postgres) | 5432 | — |

Verify the API:

```bash
curl http://localhost:8080/api/health
# => {"status":"ok","git_sha":"..."}
```

## Development (without Docker)

**Backend:**

```bash
cd backend
cargo build
DATABASE_URL=postgres://localhost/gamblingfinance cargo run
```

**Frontend:**

```bash
cd frontend
npm install
npm run dev                     # http://localhost:5173, proxies /api to :8080
npm run gen:api                 # regen src/types/api.ts from OpenAPI
```

**Migrations** are in [`migrations/`](./migrations/). Apply with
`sqlx migrate run` from `backend/` (requires `sqlx-cli`):

```bash
cargo install sqlx-cli --no-default-features --features postgres,rustls
cd backend && sqlx migrate run
```

## Repo map

```
backend/          # Rust Axum + sqlx crate
frontend/         # Vue 3 + Vite + TS app
migrations/       # sqlx SQL migrations
docs/             # agents, style guide, ADRs, OpenAPI contract
docker/           # Dockerfiles + compose.yaml
plan.md           # full design
```

## Contributing

This repo is built by multiple agents working in parallel. Before touching
anything, read [`docs/agents.md`](./docs/agents.md) to find your lane and
[`CLAUDE.md`](./CLAUDE.md) for the top-level rules.
