use pretty_assertions::assert_eq;
use rstest::rstest;

use crate::utils::is_subsequence;

#[rstest]
#[case::empty(
    &[],
    &[],
    true
)]
#[case::empty_subsequence(
    &[],
    &["a", "b"],
    true
)]
#[case::empty_sequence(
    &["a"],
    &[],
    false
)]
#[case::subsequence_1(
    &["a"],
    &["a", "b", "c"],
    true
)]
#[case::subsequence_2(
    &["b"],
    &["a", "b", "c"],
    true
)]
#[case::subsequence_3(
    &["c"],
    &["a", "b", "c"],
    true
)]
#[case::subsequence_4(
    &["a", "b"],
    &["a", "b", "c"],
    true
)]
#[case::subsequence_5(
    &["a", "c"],
    &["a", "b", "c"],
    true
)]
#[case::subsequence_6(
    &["b", "c"],
    &["a", "b", "c"],
    true
)]
#[case::subsequence_7(
    &["a", "b", "c"],
    &["a", "b", "c"],
    true
)]
#[case::out_of_order_1(
    &["b", "a"],
    &["a", "b", "c"],
    false
)]
#[case::out_of_order_2(
    &["b", "a", "c"],
    &["a", "b", "c"],
    false
)]
#[case::unrelated(
    &["a", "b", "d"],
    &["a", "b", "c"],
    false
)]
fn test_is_subsequence(
    #[case] subsequence: &[&str],
    #[case] sequence: &[&str],
    #[case] expected_result: bool,
) {
    assert_eq!(is_subsequence(subsequence, sequence), expected_result);
}

// Make sure we have the arbitrary precision feature of serde_json.
#[test]
fn serialization_precision() {
    let input =
        "{\"value\":244116128358498188146337218061232635775543270890529169229936851982759783745}";
    let serialized = serde_json::from_str::<serde_json::Value>(input).unwrap();
    let deserialized = serde_json::to_string(&serialized).unwrap();
    assert_eq!(input, deserialized);
}
