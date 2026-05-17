//! Conversions between hand-coded domain types in [`crate::feature::sync_protocol`]
//! and codegen'd wire types in [`crate::proto::sync`].
//!
//! Bridges the two type layers introduced by WT-PROTO-1. Domain types are the
//! in-memory representation used by Rust call sites today; wire types are the
//! protobuf-shaped representation used at gRPC boundaries. Once all callers
//! migrate to wire types (WT-BG-4/5/6, WT-SRV-4), the domain layer deprecates
//! and these conversions delete with it.
//!
//! Conventions:
//! - `From<DomainType> for ProtoType` for domain→wire (infallible — UUIDs
//!   always stringify; required fields always present in domain land).
//! - `TryFrom<ProtoType> for DomainType` with `Error = ConversionError` for
//!   wire→domain (fallible — wire may have None for required-in-spirit
//!   fields, invalid UUIDs, missing oneof variants, out-of-range timestamps).
//! - `#[non_exhaustive]` on the error enum to allow additive evolution.

use std::str::FromStr;

use thiserror::Error;
use uuid::Uuid;

use crate::core::hash::ContentHash;
use crate::core::id::{AccountId, BranchId, SnapshotId, TenantId, TreeId};
use crate::proto::sync as wire;

/// Error type for fallible wire→domain conversions.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ConversionError {
    /// A UUID-shaped wire string failed to parse.
    #[error("invalid UUID for `{field}`: {source}")]
    InvalidUuid {
        field: &'static str,
        #[source]
        source: uuid::Error,
    },

    /// A `bytes`-shaped wire ContentHash had the wrong length.
    #[error("invalid content hash for `{field}`: expected 32 bytes, got {actual}")]
    InvalidContentHash { field: &'static str, actual: usize },

    /// A required-in-spirit message field was None on the wire.
    /// (proto3 message fields are always `Option<T>` in prost codegen even
    /// when the .proto marks them as required.)
    #[error("required field `{0}` is missing")]
    MissingRequired(&'static str),

    /// A proto3 enum field had a value not recognized by the codegen'd enum
    /// (typically zero / UNSPECIFIED or an unknown future value).
    #[error("invalid `{enum_name}` value: {value}")]
    InvalidEnum { enum_name: &'static str, value: i32 },

    /// A google.protobuf.Timestamp had seconds/nanos out of representable range.
    #[error("invalid timestamp for `{field}`: {reason}")]
    InvalidTimestamp {
        field: &'static str,
        reason: &'static str,
    },

    /// A `oneof` wire field arrived with no variant set
    /// (`Option<...::Reason>` was None).
    #[error("oneof `{0}` has no variant set")]
    MissingOneofVariant(&'static str),
}

// ============================================================
// Helpers
// ============================================================

/// Convert a `chrono::DateTime<Utc>` into a prost-types Timestamp.
pub(crate) fn datetime_to_prost(dt: chrono::DateTime<chrono::Utc>) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}

/// Convert a prost-types Timestamp back into `chrono::DateTime<Utc>`.
pub(crate) fn prost_to_datetime(
    ts: prost_types::Timestamp,
    field: &'static str,
) -> Result<chrono::DateTime<chrono::Utc>, ConversionError> {
    chrono::DateTime::from_timestamp(ts.seconds, ts.nanos as u32).ok_or(
        ConversionError::InvalidTimestamp {
            field,
            reason: "out of range",
        },
    )
}

// ============================================================
// ID type conversions
// ============================================================

// Macro to generate the boilerplate for UUID-backed ID types.
macro_rules! impl_uuid_id_conversions {
    ($domain:ty, $wire:ty, $field_name:literal) => {
        impl From<$domain> for $wire {
            fn from(id: $domain) -> Self {
                Self {
                    value: id.as_uuid().to_string(),
                }
            }
        }

        impl TryFrom<$wire> for $domain {
            type Error = ConversionError;

            fn try_from(w: $wire) -> Result<Self, Self::Error> {
                let uuid =
                    Uuid::from_str(&w.value).map_err(|source| ConversionError::InvalidUuid {
                        field: $field_name,
                        source,
                    })?;
                Ok(<$domain>::from_uuid(uuid))
            }
        }
    };
}

impl_uuid_id_conversions!(TenantId, wire::TenantId, "TenantId.value");
impl_uuid_id_conversions!(TreeId, wire::TreeId, "TreeId.value");
impl_uuid_id_conversions!(BranchId, wire::BranchId, "BranchId.value");
impl_uuid_id_conversions!(SnapshotId, wire::SnapshotId, "SnapshotId.value");
impl_uuid_id_conversions!(AccountId, wire::AccountId, "AccountId.value");

// ContentHash is bytes-backed (32 bytes BLAKE3).
impl From<ContentHash> for wire::ContentHash {
    fn from(h: ContentHash) -> Self {
        Self {
            value: h.as_bytes().to_vec(),
        }
    }
}

impl TryFrom<wire::ContentHash> for ContentHash {
    type Error = ConversionError;

    fn try_from(w: wire::ContentHash) -> Result<Self, Self::Error> {
        let arr: [u8; 32] =
            w.value
                .as_slice()
                .try_into()
                .map_err(|_| ConversionError::InvalidContentHash {
                    field: "ContentHash.value",
                    actual: w.value.len(),
                })?;
        Ok(ContentHash::from_bytes(arr))
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::hash::hash_bytes;

    // ID roundtrips

    #[test]
    fn tenant_id_roundtrip() {
        let id = TenantId::new();
        let wire: wire::TenantId = id.into();
        let back: TenantId = wire.try_into().unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn tree_id_roundtrip() {
        let id = TreeId::new();
        let wire: wire::TreeId = id.into();
        let back: TreeId = wire.try_into().unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn branch_id_roundtrip() {
        let id = BranchId::new();
        let wire: wire::BranchId = id.into();
        let back: BranchId = wire.try_into().unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn snapshot_id_roundtrip() {
        let id = SnapshotId::new();
        let wire: wire::SnapshotId = id.into();
        let back: SnapshotId = wire.try_into().unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn account_id_roundtrip() {
        let id = AccountId::new();
        let wire: wire::AccountId = id.into();
        let back: AccountId = wire.try_into().unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn content_hash_roundtrip() {
        let h = hash_bytes(b"hello worktree");
        let wire: wire::ContentHash = h.into();
        let back: ContentHash = wire.try_into().unwrap();
        assert_eq!(h, back);
    }

    // Error cases

    #[test]
    fn tenant_id_invalid_uuid_error() {
        let wire = wire::TenantId {
            value: "not-a-uuid".into(),
        };
        let err = TenantId::try_from(wire).unwrap_err();
        match err {
            ConversionError::InvalidUuid { field, .. } => {
                assert_eq!(field, "TenantId.value");
            }
            _ => panic!("expected InvalidUuid, got {err:?}"),
        }
    }

    #[test]
    fn content_hash_wrong_length_error() {
        let wire = wire::ContentHash {
            value: vec![0u8; 31],
        };
        let err = ContentHash::try_from(wire).unwrap_err();
        match err {
            ConversionError::InvalidContentHash { field, actual } => {
                assert_eq!(field, "ContentHash.value");
                assert_eq!(actual, 31);
            }
            _ => panic!("expected InvalidContentHash, got {err:?}"),
        }
    }

    // Timestamp helpers

    #[test]
    fn datetime_roundtrip() {
        let now = chrono::Utc::now();
        let prost = datetime_to_prost(now);
        let back = prost_to_datetime(prost, "test").unwrap();
        // chrono nanos precision is preserved
        assert_eq!(now, back);
    }
}
