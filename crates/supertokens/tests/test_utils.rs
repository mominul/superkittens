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
    assert_eq!(supertokens::utils::normalise_http_method("DeLeTe"), "delete");
}
