use super::*;

#[test]
fn launch_commit_names_shortfalls_and_defaults_without_blocking_risk() {
    assert_eq!(launch_commit_label(0, 0), "[ LAUNCH ]");
    assert_eq!(launch_commit_label(0, 2), "LAUNCH UNDERSTOCKED · 2");
    assert_eq!(launch_commit_label(1, 0), "LAUNCH & DEFAULT 1");
    assert_eq!(launch_commit_label(2, 3), "LAUNCH · 3 SHORT · DEFAULT 2");
}
