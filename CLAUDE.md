# Agent guidance — TheGamblingFinance

**Read first:** [`plan.md`](./plan.md) (big picture) →
[`docs/agents.md`](./docs/agents.md) (your lane) →
[`docs/style-guide.md`](./docs/style-guide.md) (if you touch UI).

## The one rule

**Stay in your lane.** Five roles (Backend-Core, Backend-Auth,
Frontend-Shell, Frontend-Features, DevOps) each own a disjoint set of files
listed in `docs/agents.md`. Do not edit files outside your lane. If you
genuinely need a change outside your lane, open an issue or PR that the
owning role reviews — do not silently cross the boundary.

## Shared contracts — changes only by handshake

These four artifacts are consumed by every other agent. Touch them only
with an explicit agreement in the PR description naming the affected roles:

1. `migrations/*.sql` — **never edit in place.** New migrations only, named
   `YYYYMMDDHHMMSS_<initials>_<slug>.sql`.
2. `docs/contracts/openapi.json` — backend PR regenerates it; frontend PR
   runs `npm run gen:api` and commits `src/types/api.ts`. CI will fail on
   drift.
3. `frontend/src/styles/tokens.css` + `docs/style-guide.md` — only
   Frontend-Shell.
4. `backend/src/domain/settlement.rs` **signature** — if it changes, the
   frontend's parity test breaks; coordinate with Frontend-Features.

## House rules

- **Money is `BIGINT` cents.** No floats. Ever.
- **Dark-theme, German locale** (`de-DE`, `Europe/Berlin`, EUR). Do not
  hardcode locale strings; use `Intl`.
- **No hex colors outside `frontend/src/styles/`.** Use CSS variables or
  Tailwind classes.
- **Tabular numerals** for anything monetary (`.tabular` utility class).
- **Commits**: Conventional Commits scoped to your lane, e.g.
  `feat(backend/nights): add close endpoint`,
  `feat(frontend/shell): bottom sheet focus trap`.
- **Branches**: `<agent>/<feature>` off `main`, rebased daily, squash-merge.
- **PRs must list** files touched + lane confirmation + any shared-contract
  handshake.

## How to run things

- Backend build: `cd backend && cargo build`
- Frontend build: `cd frontend && npm install && npm run build`
- Full stack: `docker compose -f docker/compose.yaml up --build`
- Regenerate frontend types: `cd frontend && npm run gen:api`

When in doubt, re-read `plan.md`. It is the tiebreaker.
