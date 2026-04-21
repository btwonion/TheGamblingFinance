# API reference

**Source of truth:** [`contracts/openapi.json`](./contracts/openapi.json).

This document is a human-friendly index. For request/response shapes,
read the OpenAPI file (or the generated `frontend/src/types/api.ts`).

## Lifecycle of this file

- **Phase 0** — `contracts/openapi.json` is **hand-written**. It covers
  every route in `plan.md` §"HTTP API surface" with minimal schemas, just
  enough for `openapi-typescript` to produce usable types for the
  Frontend-Shell to begin work in Phase 1.
- **Phase 1+** — Backend-Core introduces `utoipa` derive macros and a
  build step that emits `contracts/openapi.json`. From that point on, the
  file is **generated**; CI fails if a backend PR doesn't commit the
  regenerated contract or a frontend PR doesn't commit the regenerated
  types.

## Conventions

- All endpoints live under `/api`. Frontend may proxy via Vite in dev.
- All request/response bodies are JSON.
- Errors follow `{ "error": { "code": "snake_case", "message": "human text", "details": {...}? } }`.
- Auth via session cookie `gf_sid`:
  - `HttpOnly; Secure; SameSite=Lax; Path=/`
  - 30-day rolling expiry (server extends on each request).
- Money is always **cents** (integer). Client formats via `Intl` in `de-DE`.

## Surface (summary)

See `contracts/openapi.json` for the full, machine-readable version.

| Method | Path | Auth |
|---|---|---|
| GET | `/api/health` | public |
| POST | `/api/auth/login` | public, rate-limited |
| POST | `/api/auth/logout` | session |
| GET | `/api/auth/me` | session |
| GET | `/api/users` | admin |
| POST | `/api/users` | admin |
| PATCH | `/api/users/:id` | admin |
| POST | `/api/users/:id/reset-password` | admin |
| PATCH | `/api/users/me` | session |
| GET | `/api/nights` | session |
| POST | `/api/nights` | admin |
| GET | `/api/nights/:id` | member or admin |
| PATCH | `/api/nights/:id` | admin, open only |
| DELETE | `/api/nights/:id` | admin, open only |
| POST | `/api/nights/:id/players` | admin, open only |
| DELETE | `/api/nights/:id/players/:uid` | admin, open only |
| POST | `/api/nights/:id/buy-ins` | admin |
| DELETE | `/api/nights/:id/buy-ins/:id` | admin, open only |
| POST | `/api/nights/:id/trades` | admin |
| DELETE | `/api/nights/:id/trades/:id` | admin, open only |
| PUT | `/api/nights/:id/cash-outs/:uid` | admin |
| POST | `/api/nights/:id/close` | admin |
| POST | `/api/nights/:id/reopen` | admin |
| GET | `/api/nights/:id/settlement` | member or admin |
| GET | `/api/nights/:id/leaderboard` | member or admin |
| GET | `/api/leaderboard` | session |
