use noesis_vedic_api::{CachedVedicClient, Config};

// Panchang has been computed natively since PR2, so these check the cache
// counters instead of counting requests to a mock vendor.

#[tokio::test]
async fn test_cached_panchang_uses_cache_for_same_date_location() {
    let config = Config::new("test_key").with_base_url("http://127.0.0.1:1");
    let client = CachedVedicClient::new(config);

    let first = client
        .get_panchang(2024, 1, 1, 12, 0, 0, 12.97, 77.59, 5.5)
        .await
        .expect("panchang first call");

    let second = client
        .get_panchang(2024, 1, 1, 13, 30, 0, 12.97, 77.59, 5.5)
        .await
        .expect("panchang second call");

    let stats = client.cache_stats().await;
    assert_eq!(stats.panchang_entries, 1, "one cached day");
    assert_eq!(stats.hits, 1, "expected cached response on second call");
    assert_eq!(first.tithi.number, second.tithi.number);
}

#[tokio::test]
async fn test_cached_panchang_diff_date_triggers_new_request() {
    let config = Config::new("test_key").with_base_url("http://127.0.0.1:1");
    let client = CachedVedicClient::new(config);

    let _first = client
        .get_panchang(2024, 1, 1, 12, 0, 0, 12.97, 77.59, 5.5)
        .await
        .expect("panchang first call");

    let _second = client
        .get_panchang(2024, 1, 2, 12, 0, 0, 12.97, 77.59, 5.5)
        .await
        .expect("panchang second call");

    let stats = client.cache_stats().await;
    assert_eq!(
        stats.panchang_entries, 2,
        "expected new entry for different date"
    );
    assert_eq!(stats.hits, 0);
}
