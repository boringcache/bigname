use serde_json::json;
use sqlx::types::time::OffsetDateTime;

use super::*;

fn scope() -> SnapshotSelectionScope {
    SnapshotSelectionScope::new(
        vec![SnapshotPositionRequirement::new(
            "ethereum",
            "ethereum-mainnet",
        )],
        Some("ethereum".to_owned()),
    )
    .expect("test scope must be valid")
}

#[test]
fn explicit_chain_positions_reject_duplicate_slots() {
    let error = ChainPositions::parse_explicit_json(
        r#"{
                "ethereum": {
                    "chain_id": "ethereum-mainnet",
                    "block_number": 1,
                    "block_hash": "0x1",
                    "timestamp": "2026-04-17T00:00:01Z"
                },
                "ethereum": {
                    "chain_id": "ethereum-mainnet",
                    "block_number": 2,
                    "block_hash": "0x2",
                    "timestamp": "2026-04-17T00:00:02Z"
                }
            }"#,
        &scope(),
    )
    .expect_err("duplicate slots must be invalid");

    assert_eq!(error.kind(), SnapshotSelectionErrorKind::InvalidInput);
    assert!(error.message().contains("repeats position slot ethereum"));
}

#[test]
fn explicit_chain_positions_reject_missing_and_wrong_profile_slots() {
    let missing = ChainPositions::parse_explicit_json("{}", &scope())
        .expect_err("missing required slot must be invalid");
    assert_eq!(missing.kind(), SnapshotSelectionErrorKind::InvalidInput);

    let wrong_chain = ChainPositions::parse_explicit_json(
        r#"{
                "ethereum": {
                    "chain_id": "ethereum-sepolia",
                    "block_number": 1,
                    "block_hash": "0x1",
                    "timestamp": "2026-04-17T00:00:01Z"
                }
            }"#,
        &scope(),
    )
    .expect_err("mixed profile chain must be invalid");
    assert_eq!(wrong_chain.kind(), SnapshotSelectionErrorKind::InvalidInput);
    assert!(wrong_chain.message().contains("expected ethereum-mainnet"));
}

#[test]
fn projection_chain_positions_match_by_chain_identity() {
    let selected = ChainPositions::from_value(&json!({
        "ethereum": {
            "chain_id": "ethereum-mainnet",
            "block_number": 7,
            "block_hash": "0x7",
            "timestamp": "2026-04-17T00:00:07Z"
        }
    }))
    .expect("selected positions must decode");
    let projected = json!({
        "ethereum-mainnet": {
            "chain_id": "ethereum-mainnet",
            "block_number": 7,
            "block_hash": "0x7",
            "timestamp": "2026-04-17T00:00:07Z"
        }
    });

    ensure_projection_chain_positions_match("name_current", &projected, &selected)
        .expect("slot aliases with the same chain identity should match");

    let stale = ensure_projection_chain_positions_match(
        "name_current",
        &json!({
            "ethereum": {
                "chain_id": "ethereum-mainnet",
                "block_number": 8,
                "block_hash": "0x8",
                "timestamp": "2026-04-17T00:00:08Z"
            }
        }),
        &selected,
    )
    .expect_err("different chain position must be stale");
    assert_eq!(stale.kind(), SnapshotSelectionErrorKind::Stale);
}

#[test]
fn rfc3339_timestamp_offsets_are_equal_instants() {
    let expected = parse_rfc3339_utc_timestamp("2025-06-15T15:07:42Z")
        .expect("canonical UTC timestamp must decode");

    for timestamp in [
        "2025-06-15T15:07:42+00:00",
        "2025-06-15T17:37:42+02:30",
        "2025-06-15T10:07:42-05:00",
    ] {
        assert_eq!(
            parse_rfc3339_utc_timestamp(timestamp).expect("numeric offset must decode"),
            expected,
            "{timestamp} must identify the same UTC instant"
        );
    }
}

#[test]
fn rfc3339_timestamp_fractional_offset_preserves_nanoseconds() {
    let expected = parse_rfc3339_utc_timestamp("2025-06-15T15:07:42.123456789Z")
        .expect("fractional UTC timestamp must decode");
    let with_offset = parse_rfc3339_utc_timestamp("2025-06-15T15:07:42.123456789+00:00")
        .expect("fractional timestamp with numeric offset must decode");

    assert_eq!(with_offset, expected);
    assert_eq!(with_offset.nanosecond(), 123_456_789);
}

#[test]
fn chain_position_serialization_preserves_fractional_seconds() {
    let timestamp = OffsetDateTime::from_unix_timestamp(1_750_000_062)
        .expect("fixture timestamp must be representable")
        .replace_nanosecond(123_456_789)
        .expect("fixture fraction must be valid");
    let position = ChainPosition {
        slot: "ethereum".to_owned(),
        chain_id: "ethereum-mainnet".to_owned(),
        block_number: 27,
        block_hash: "0x4a98".to_owned(),
        timestamp,
    };

    assert_eq!(
        position.to_value()["timestamp"],
        "2025-06-15T15:07:42.123456789Z"
    );
}
