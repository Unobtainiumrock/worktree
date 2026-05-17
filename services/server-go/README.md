# worktree-server (Go)

**Remote multi-tenant server** for the W0rkTree VCS. Runs in the cloud
(or on-prem), holds canonical history, enforces IAM, handles chunk
routing, and aggregates staged snapshots from `worktree-bg` clients.

This is **not** the local daemon — that's `crates/worktree-bg/` (Rust).
The current `crates/worktree-server/` Rust crate is transitional and
slated for removal once this Go implementation reaches feature parity
(per the 2026-05-15 language-pivot decision: Rust for the local bg,
Go for the remote server).

## Status

Skeleton only — module declaration + stub `main.go` that prints
`worktree-server: not yet implemented (skeleton — WT-SRV-1)` and exits.
Real implementation work is queued:

- **WT-PROTO-1** — Adopt protobuf wire format (decision: Option A,
  taken 2026-05-17). Gates WT-SRV-2.
- **WT-SRV-2** — Codegen gRPC service from
  `crates/worktree-protocol/` proto definitions (blocked on WT-PROTO-1).
- **WT-SRV-3** — Object store: content-addressable BLAKE3 + dedup +
  per-tenant namespacing per Server.md §14.
- **WT-SRV-4** — Single-tenant sync handlers: stage, push, pull,
  config-sync, tag-push.
- **WT-SRV-5** — `/health` + `/metrics` (Prometheus) endpoints per
  Server.md §20.

## Spec

`../../crates/worktree-protocol/specs/server/Server.md`

## Prerequisites

- Go 1.22 or later
- [`golangci-lint`](https://golangci-lint.run/) — install once, used
  by `scripts/ci.sh`

## Build + run

```bash
cd services/server-go
go build ./...
./server-go
```

Output:

```
worktree-server: not yet implemented (skeleton — WT-SRV-1)
```
