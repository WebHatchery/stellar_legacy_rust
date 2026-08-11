use super::*;

#[test]
fn the_command_header_counts_in_words_that_agree() {
    assert_eq!(plural(1, "captain", "captains"), "1 captain");
    assert_eq!(plural(0, "captain", "captains"), "0 captains");
    assert_eq!(plural(4, "captain", "captains"), "4 captains");
}
