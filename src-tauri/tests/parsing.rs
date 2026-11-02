//! Tests for the parsing helpers that turn engine output into UI numbers.

use better_mtplx_ui_lib::instances::discovery::{PackSize, resolve_weights};
use better_mtplx_ui_lib::instances::manager;

/// The engine's memory-plan line, verbatim from `mtplx serve`.
const MEMORY_PLAN: &str = "[5/6] Memory plan: 256G Mac: engine budget 192.0G, weights 77.3G, context 262144, session bank up to 48.0G, n-gram table 29.8G streamed from SSD (not wired)";

#[test]
fn weights_are_read_from_the_memory_plan_line() {
    let weights = manager::parse_weights_bytes(MEMORY_PLAN).expect("weights");
    assert_eq!(weights, (77.3 * 1024.0 * 1024.0 * 1024.0) as u64);
}

#[test]
fn unrelated_log_lines_have_no_weights() {
    assert_eq!(
        manager::parse_weights_bytes("[5/6] Model loaded in 4.1s"),
        None
    );
    assert_eq!(manager::parse_weights_bytes(""), None);
}

#[test]
fn size_tokens_parse_in_every_unit() {
    assert_eq!(manager::parse_size_token("1G"), Some(1024 * 1024 * 1024));
    assert_eq!(manager::parse_size_token("1GiB"), Some(1024 * 1024 * 1024));
    assert_eq!(manager::parse_size_token("512M"), Some(512 * 1024 * 1024));
    assert_eq!(
        manager::parse_size_token("1.5G"),
        Some((1.5 * 1024.0 * 1024.0 * 1024.0) as u64)
    );
    assert_eq!(manager::parse_size_token("nope"), None);
}

#[test]
fn external_server_weights_resolve_from_the_cache() {
    let packs = vec![
        PackSize {
            repo_id: "Youssofal/Qwen3.8-27B-MTPLX-Optimized-Quality".to_string(),
            size_bytes: 29_972_716_973,
        },
        PackSize {
            repo_id: "Youssofal/Qwen3.8-Flash-Next-MTPLX-Optimized-Speed".to_string(),
            size_bytes: 107_000_000_000,
        },
    ];

    let quality = resolve_weights("mtplx-qwen38-27b-optimized-quality", &packs);
    assert_eq!(quality, Some(29_972_716_973));

    let speed = resolve_weights("mtplx-flash-next-optimized-speed", &packs);
    assert_eq!(speed, Some(107_000_000_000));

    // A model we have never cached has no resolvable size.
    assert_eq!(resolve_weights("mtplx-some-unknown-model", &packs), None);
}
