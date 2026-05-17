//! Protobuf-generated wire types and gRPC service definitions.
//!
//! Generated from `proto/sync.proto` via `build.rs` (tonic-build).
//! Coexists with the hand-coded Rust domain types in
//! [`crate::feature::sync_protocol`] during the WT-PROTO migration —
//! conversion impls between domain and wire types land in WT-PROTO-2.

pub mod sync {
    tonic::include_proto!("worktree.sync.v1");
}
