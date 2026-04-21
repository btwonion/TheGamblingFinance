# Agent ownership & working agreement

This repo is built by multiple agents working in parallel. Ownership is by
**file path**, not by feature. If your change touches a path you do not
own, stop and coordinate with the owning role via PR description or issue.

> **Before you start:** read `plan.md` (the big picture), this file
> (your lane), and `docs/style-guide.md` if you touch UI.

---

## Roles and file ownership

| Agent | Owns | Depends on |
|---|---|---|
| **Backend-Core** | `backend/src/{db,domain,dto,routes}/{nights,buy_ins,trades,cash_outs,settlement,leaderboard,users}.rs` and settlement tests | Phase 0 migrations; Backend-Auth's `AuthedUser` + `RequireAdmin` extractors |
| **Backend-Auth** | `backend/src/{routes,middleware,db}/auth*.rs`, `backend/src/util/{password,cookies}.rs`, rate-limit middleware | `users` + `auth_sessions` migrations |
| **Frontend-Shell** | `frontend/src/{main.ts,App.vue,router.ts,api/client.ts,styles/**}`, all generic components in `frontend/src/components/`, `frontend/src/stores/auth.ts`, `frontend/src/views/LoginView.vue`, dev-only `/_ui` style-guide demo | Phase 0 OpenAPI skeleton |
| **Frontend-Features** | `frontend/src/views/*` (except `LoginView.vue`), `frontend/src/stores/{nights,players,leaderboard}.ts`, `frontend/src/api/{nights,players,leaderboard}.ts` | Frontend-Shell components, Backend-Core endpoints |
| **DevOps** | `docker/**`, `.env.example`, migration runner, this file (structural updates), optional CI under `.github/workflows/` | Buildable backend + frontend |

Phase 0 itself was written by a single scaffolding pass and merged to
`main` before any parallel work began.

---

## Shared contracts — changes only by handshake

Four artifacts are consumed by more than one role. Any change to them
must be called out in the PR description and named-reviewed by the
affected roles.

1. **Migrations** — `migrations/*.sql`. Never edit in place. New
   migrations only, named `YYYYMMDDHHMMSS_<initials>_<slug>.sql`. CI
   rejects duplicate timestamps.
2. **API contract** — `docs/contracts/openapi.json`. Phase 0 ships a
   hand-written skeleton; once Backend-Core wires `utoipa`, the file is
   regenerated on every backend build. The frontend commits
   `src/types/api.ts` via `npm run gen:api`. CI will fail on drift.
3. **Style tokens** — `frontend/src/styles/tokens.css` and
   `docs/style-guide.md`. Only Frontend-Shell edits these.
4. **Settlement algorithm** — signature of `settle(...)` in
   `backend/src/domain/settlement.rs`. The frontend parity test compiles
   against the serde types this produces; changing it breaks the
   frontend. Pseudocode and determinism contract live in
   `docs/adr/0002-iou-model.md` and are the tiebreaker.

---

## Branching and commits

- **Branch name:** `<agent>/<feature>` off `main`. Examples:
  `backend-core/close-endpoint`, `frontend-shell/bottom-sheet`.
- **Rebase daily** to keep divergence small; **squash-merge** to `main`.
- **Conventional Commits** scoped to your lane:

    | Scope | Example |
    |---|---|
    | `backend/nights` | `feat(backend/nights): add close endpoint` |
    | `backend/auth` | `feat(backend/auth): argon2id password hasher` |
    | `backend/settlement` | `fix(backend/settlement): break ties by user_id bytes` |
    | `frontend/shell` | `feat(frontend/shell): BottomSheet focus trap` |
    | `frontend/features` | `feat(frontend/features): nights list view` |
    | `db` | `feat(db): add settlement_transfers.seq index` |
    | `docker` | `feat(docker): multi-stage backend image` |
    | `docs` | `docs(adr): add 0003-session-rotation` |

- **No hooks skipped.** If a pre-commit or CI check fails, fix the cause,
  don't bypass. `--no-verify` is never okay.

---

## PR checklist

Every PR body must include:

- [ ] **Lane**: which agent role wrote this change
- [ ] **Files touched**: top-level list (copy-paste from `git diff --name-only`)
- [ ] **Shared-contract handshake?** If yes, name the affected roles and
      link the coordinating issue/PR.
- [ ] **Migration?** If yes, include the migration filename and a one-line
      description of the schema change.
- [ ] **API change?** If yes, confirm `openapi.json` was regenerated and
      `src/types/api.ts` updated on the frontend side (or file an issue
      for the counterpart agent).
- [ ] **Tests**: list what you added/modified.

---

## Escalation and boundary crossing

- **Need a generic component that doesn't exist?** Frontend-Features must
  file a `shell:gap` issue describing the need; Frontend-Shell adds it.
  Do **not** create bespoke components inside `views/`.
- **Need a new backend endpoint?** Frontend-Features opens an issue
  tagged `api:request`; Backend-Core adds it.
- **Shared-contract emergency?** If a contract change blocks your work
  and the owning role is unavailable, write a draft PR that makes the
  minimal change, tag the owner, and **do not merge** until they
  approve.
- **Disagreement about scope?** `plan.md` is the tiebreaker. If the plan
  is silent, open an ADR under `docs/adr/` proposing the decision.
