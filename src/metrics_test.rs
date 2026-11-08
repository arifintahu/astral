use super::*;

#[test]
fn top_by_returns_the_highest_first() {
    let items = vec![
        ("a", 5.0),
        ("b", 90.0),
        ("c", 1.0),
        ("d", 40.0),
        ("e", 70.0),
    ];
    let top = top_by(items, 3, |(_, v)| *v);
    assert_eq!(
        top.iter().map(|(n, _)| *n).collect::<Vec<_>>(),
        ["b", "e", "d"]
    );
}

#[test]
fn top_by_keeps_everything_when_there_are_fewer_than_n() {
    let top = top_by(vec![("a", 1.0), ("b", 2.0)], 15, |(_, v)| *v);
    assert_eq!(top.iter().map(|(n, _)| *n).collect::<Vec<_>>(), ["b", "a"]);
}

#[test]
fn top_by_handles_empty_input_and_nan() {
    assert!(top_by(Vec::<f32>::new(), 15, |v| *v).is_empty());
    let top = top_by(vec![f32::NAN, 2.0, 1.0], 2, |v| *v);
    assert_eq!(top.len(), 2);
}

#[test]
fn per_second_scales_a_delta_by_the_elapsed_time() {
    assert_eq!(per_second(1000, 1.0), 1000);
    assert_eq!(per_second(10_000, 10.0), 1000);
    assert_eq!(per_second(500, 0.5), 1000);
}

#[test]
fn per_second_never_divides_by_zero() {
    assert!(per_second(100, 0.0) > 0);
}

#[test]
fn samples_every_second_only_while_a_dashboard_is_connected() {
    // The worker is always subscribed; a dashboard is anything beyond it.
    assert_eq!(sample_interval(1), IDLE_EVERY);
    assert_eq!(sample_interval(2), WATCHED_EVERY);
    assert_eq!(sample_interval(5), WATCHED_EVERY);
}
