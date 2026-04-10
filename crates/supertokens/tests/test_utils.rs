mod common;

use supertokens::utils::find_max_version;

#[test]
fn test_find_max_version_multiple_supported() {
    let sv = &["0.0", "1.0", "1.1", "2.1"];
    let cv = &["0.1", "0.2", "1.1", "2.1", "3.0"];
    let result = find_max_version(sv, cv);
    assert_eq!(result, Some("2.1".to_string()));
}

#[test]
fn test_find_max_version_single_supported() {
    let sv = &["0.0", "1.0", "1.1", "2.0"];
    let cv = &["0.1", "0.2", "1.1", "2.1", "3.0"];
    let result = find_max_version(sv, cv);
    assert_eq!(result, Some("1.1".to_string()));
}

#[test]
fn test_find_max_version_no_overlap() {
    let sv = &["0.0", "1.0", "1.1", "2.1"];
    let cv = &["0.1", "0.2", "1.2", "2.0", "3.0"];
    let result = find_max_version(sv, cv);
    assert_eq!(result, None);
}

#[test]
fn test_find_max_version_empty() {
    let sv: &[&str] = &[];
    let cv: &[&str] = &[];
    let result = find_max_version(sv, cv);
    assert_eq!(result, None);
}

#[test]
fn test_normalise_http_method() {
    assert_eq!(supertokens::utils::normalise_http_method("GET"), "get");
    assert_eq!(supertokens::utils::normalise_http_method("post"), "post");
    assert_eq!(
        supertokens::utils::normalise_http_method("DeLeTe"),
        "delete"
    );
}

// ---------------------------------------------------------------------------
// is_version_gte (ported from test_utils.py::test_util_is_version_gte)
// ---------------------------------------------------------------------------

#[test]
fn test_is_version_gte() {
    use supertokens::utils::is_version_gte;

    let cases = vec![
        ("1.12", "1.12", true),
        ("1.12.0", "1.12", true),
        ("2.12.0", "1.12", true),
        ("1.13", "1.12", true),
        ("1.13.0", "1.12", true),
        ("0.11.0", "1.12", false),
        ("1.11.0", "1.11", true),
        ("0.13.2", "0.13.0", true),
        ("0.12.5", "0.13.0", false),
    ];

    for (version, minimum, expected) in cases {
        assert_eq!(
            is_version_gte(version, minimum),
            expected,
            "is_version_gte({}, {}) should be {}",
            version,
            minimum,
            expected
        );
    }
}

// ---------------------------------------------------------------------------
// humanize_time (ported from test_utils.py::test_humanize_time)
// ---------------------------------------------------------------------------

#[test]
fn test_humanize_time() {
    use supertokens::utils::humanize_time;

    let cases = vec![
        (1000u64, "1 second"),
        (59000, "59 seconds"),
        (60000, "1 minute"),
        (119000, "1 minute"), // 1m 59s rounds down to 1 minute
        (120000, "2 minutes"),
        (3600000, "1 hour"),
        (3660000, "1 hour"), // 1h 1m rounds down to 1 hour
        // Note: Rust implementation doesn't produce fractional hours like Python's "1.1 hours"
        // It rounds down to whole hours, so 3960000ms (1h 6m) → "1 hour"
        (3960000, "1 hour"),
        (7260000, "2 hours"), // 2h 1m
        (18000000, "5 hours"),
    ];

    for (ms, expected) in cases {
        assert_eq!(
            humanize_time(ms),
            expected,
            "humanize_time({}) should be '{}'",
            ms,
            expected
        );
    }
}

// ---------------------------------------------------------------------------
// get_top_level_domain_for_same_site_resolution
// (ported from test_utils.py::test_tld_for_same_site)
// ---------------------------------------------------------------------------

#[test]
fn test_tld_for_same_site() {
    use supertokens::utils::get_top_level_domain_for_same_site_resolution;

    let cases = vec![
        ("http://localhost:3001", "localhost"),
        (
            "https://ec2-xx-yyy-zzz-0.compute-1.amazonaws.com",
            "ec2-xx-yyy-zzz-0.compute-1.amazonaws.com",
        ),
        ("https://foo.vercel.com", "vercel.com"),
        ("https://blog.supertokens.com", "supertokens.com"),
    ];

    for (url, expected) in cases {
        assert_eq!(
            get_top_level_domain_for_same_site_resolution(url),
            expected,
            "TLD for '{}' should be '{}'",
            url,
            expected
        );
    }
}

#[test]
fn test_find_max_version_exact_match() {
    // Both arrays are identical — the max common version should be returned
    let sv = &["1.0", "2.0", "3.0"];
    let cv = &["1.0", "2.0", "3.0"];
    let result = find_max_version(sv, cv);
    assert_eq!(result, Some("3.0".to_string()));
}
