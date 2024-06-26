use pretty_assertions::assert_eq;
use rstest::rstest;

use crate::utils::is_subsequence;

#[rstest]
#[case::empty(&[], &[], true)]
#[case::empty_subsequence(&[], &["a".to_string(), "b".to_string()], true)]
#[case::empty_sequence(&["a".to_string()], &[], false)]
#[case::subsequence_1(
    &["a".to_string()],
    &["a".to_string(), "b".to_string(), "c".to_string()],
    true
)]
#[case::subsequence_1(
    &["b".to_string()],
    &["a".to_string(), "b".to_string(), "c".to_string()],
    true
)]
#[case::subsequence_1(
    &["c".to_string()],
    &["a".to_string(), "b".to_string(), "c".to_string()],
    true
)]
#[case::subsequence_4(
    &["a".to_string(), "b".to_string()],
    &["a".to_string(), "b".to_string(), "c".to_string()],
    true
)]
#[case::subsequence_5(
    &["a".to_string(), "c".to_string()],
    &["a".to_string(), "b".to_string(), "c".to_string()],
    true
)]
#[case::subsequence_6(
    &["b".to_string(), "c".to_string()],
    &["a".to_string(), "b".to_string(), "c".to_string()],
    true
)]
#[case::subsequence_7(
    &["a".to_string(), "b".to_string(), "c".to_string()],
    &["a".to_string(), "b".to_string(), "c".to_string()],
    true
)]
#[case::out_of_order_1(
    &["b".to_string(), "a".to_string()],
    &["a".to_string(), "b".to_string(), "c".to_string()],
    false
)]
#[case::out_of_order_2(
    &["b".to_string(), "a".to_string(), "c".to_string()],
    &["a".to_string(), "b".to_string(), "c".to_string()],
    false
)]
#[case::unrelated(
    &["a".to_string(), "b".to_string(), "d".to_string()],
    &["a".to_string(), "b".to_string(), "c".to_string()],
    false
)]
fn test_is_subsequence(
    #[case] subsequence: &[String],
    #[case] sequence: &[String],
    #[case] expected_result: bool,
) {
    assert_eq!(is_subsequence(subsequence, sequence), expected_result);
}
