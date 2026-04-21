# ADR 0001 — Single-crate backend

- **Status:** accepted
- **Date:** 2026-04-21
- **Deciders:** anton

## Context

The backend surface is modest: one HTTP server, one binary, one database.
The design in `plan.md` lists roughly eight route modules, a pure domain
module (`settlement.rs`), a handful of middleware, and a shared `sqlx` pool.
There is no WebAssembly target, no shared library for an external crate to
consume, and no plan to split the runtime into separable services.

## Decision

Ship the backend as a **single Cargo crate** (`backend/`) rather than a
Cargo workspace with multiple member crates. One `Cargo.toml`, one
`src/main.rs` (binary), and an optional `src/lib.rs` so integration tests
in `tests/` can link against internal modules.

## Rationale

- **Compile-time checks compose better in one crate.** `sqlx::query!`
  macros read query metadata from a single `.sqlx/` directory; splitting
  the crate would fragment prepared-query cache layout.
- **Refactors are cheap inside a crate, expensive across crates.** At this
  phase we are still discovering the domain; module boundaries will move
  as we learn. Moving code between crates requires touching `Cargo.toml`
  files and `pub` surfaces; moving between modules of one crate is a file
  move.
- **Faster cold compile.** One dep graph, one `target/` cache.
- **Simpler Docker build.** One `COPY backend/`, one `cargo build --release`.
- **No multi-team friction.** The five-agent plan scopes owners by *file
  path*, not by crate. A workspace would add ceremony without adding
  isolation.

## Consequences

- If the domain layer ever needs to be consumed by an external tool (CLI
  auditor, reporting job), we will extract a `domain` crate at that time
  — cheaper to do when the shape is known than to pre-optimize now.
- All tests live in `backend/tests/` and compile against `lib.rs` exports.
- `cargo check` and `cargo clippy` at the repo root still work; CI can
  run them from `backend/` with a single step.

## Alternatives considered

- **Workspace with `api`, `domain`, `db` crates.** Rejected: no crate
  would have an external consumer, and the split would front-load
  refactoring cost.
- **Binary-only, no `lib.rs`.** Rejected: `#[sqlx::test]` integration
  tests and the `proptest` settlement tests need to reach internal types.
