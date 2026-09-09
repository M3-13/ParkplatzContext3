use parkplatz_lib::watcher::parse_branch_from_head;

#[test]
fn parses_symbolic_ref_branch() {
    assert_eq!(
        parse_branch_from_head("ref: refs/heads/main\n"),
        Some("main".to_string())
    );
}

#[test]
fn parses_branch_with_slashes() {
    assert_eq!(
        parse_branch_from_head("ref: refs/heads/feature/login-flow\n"),
        Some("feature/login-flow".to_string())
    );
}

#[test]
fn parses_branch_without_trailing_newline() {
    assert_eq!(
        parse_branch_from_head("ref: refs/heads/dev"),
        Some("dev".to_string())
    );
}

#[test]
fn returns_none_for_detached_head() {
    assert_eq!(
        parse_branch_from_head("deadbeefdeadbeefdeadbeefdeadbeefdeadbeef\n"),
        None
    );
}

#[test]
fn returns_none_for_empty_or_unrelated_content() {
    assert_eq!(parse_branch_from_head(""), None);
    assert_eq!(parse_branch_from_head("ref: refs/heads/"), None);
    assert_eq!(parse_branch_from_head("not a head"), None);
}
