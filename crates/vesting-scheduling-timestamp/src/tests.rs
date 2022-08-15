//! Tests.

use pallet_vesting::traits::SchedulingDriver;

use super::*;
use crate::mock::*;

#[test]
fn multi_linear_parsing() {
    let tests = [
        (r#"[]"#, vec![]),
        (
            r#"[{"balance":10,"cliff":10,"vesting":10}]"#,
            vec![LinearSchedule {
                balance: 10,
                cliff: 10,
                vesting: 10,
            }],
        ),
        (
            r#"[{"balance":20,"cliff":30,"vesting":40},{"balance":50,"cliff":60,"vesting":70}]"#,
            vec![
                LinearSchedule {
                    balance: 20,
                    cliff: 30,
                    vesting: 40,
                },
                LinearSchedule {
                    balance: 50,
                    cliff: 60,
                    vesting: 70,
                },
            ],
        ),
    ];

    for (input, expected) in tests {
        let expected: MultiLinearScheduleOf<Test> = expected.try_into().unwrap();
        let actual: MultiLinearScheduleOf<Test> = serde_json::from_str(input).unwrap();
        assert_eq!(actual, expected);
    }
}

#[test]
fn multi_linear_parsing_no_unknown() {
    let input = r#"[{"balance":10,"cliff":10,"vesting":10,"unknown_field":123}]"#;
    let err = serde_json::from_str::<MultiLinearScheduleOf<Test>>(input).unwrap_err();
    assert_eq!(
        err.to_string(),
        "unknown field `unknown_field`, expected one of \
        `balance`, `cliff`, `vesting` at line 1 column 54"
    )
}

#[test]
fn multi_linear_parsing_too_many_schedules() {
    let input = r#"[
        {"balance":1,"cliff":10,"vesting":10},
        {"balance":2,"cliff":10,"vesting":10},
        {"balance":3,"cliff":10,"vesting":10},
        {"balance":4,"cliff":10,"vesting":10},
        {"balance":5,"cliff":10,"vesting":10},
        {"balance":6,"cliff":10,"vesting":10}
    ]"#;
    let err = serde_json::from_str::<MultiLinearScheduleOf<Test>>(input).unwrap_err();
    assert_eq!(err.to_string(), "out of bounds at line 8 column 5")
}

fn multi_linear_schedule(
    schedule: impl IntoIterator<Item = (u8, u8, u8)>,
) -> MultiLinearScheduleOf<Test> {
    let vec: Vec<_> = schedule
        .into_iter()
        .map(|(balance, cliff, vesting)| LinearSchedule {
            balance,
            cliff,
            vesting,
        })
        .collect();
    vec.try_into().unwrap()
}

fn compute_result(
    schedule: &MultiLinearScheduleOf<Test>,
    starting_point: <Test as Config>::Timestamp,
    now: <Test as Config>::Timestamp,
) -> Result<<Test as Config>::Balance, DispatchError> {
    let lock = mocks_lock();

    let starting_point_context = MockStartingPoint::get_context();
    let now_context = MockNow::get_context();

    starting_point_context
        .expect()
        .once()
        .return_const(Some(starting_point));
    now_context.expect().once().return_const(now);

    let res = Driver::compute_balance_under_lock(schedule);

    starting_point_context.checkpoint();
    now_context.checkpoint();

    drop(lock);

    res
}

fn compute(
    schedule: &MultiLinearScheduleOf<Test>,
    starting_point: <Test as Config>::Timestamp,
    now: <Test as Config>::Timestamp,
) -> <Test as Config>::Balance {
    compute_result(schedule, starting_point, now).unwrap()
}

#[test]
fn multi_linear_logic() {
    let schedule = multi_linear_schedule([(3, 0, 0), (10, 10, 10), (100, 20, 10)]);

    assert_eq!(compute(&schedule, 20, 20), 110);
    assert_eq!(compute(&schedule, 20, 21), 110);
    assert_eq!(compute(&schedule, 20, 22), 110);
    assert_eq!(compute(&schedule, 20, 29), 110);
    assert_eq!(compute(&schedule, 20, 30), 110);
    assert_eq!(compute(&schedule, 20, 31), 109);
    assert_eq!(compute(&schedule, 20, 32), 108);
    assert_eq!(compute(&schedule, 20, 38), 102);
    assert_eq!(compute(&schedule, 20, 39), 101);
    assert_eq!(compute(&schedule, 20, 40), 100);
    assert_eq!(compute(&schedule, 20, 41), 90);
    assert_eq!(compute(&schedule, 20, 42), 80);
    assert_eq!(compute(&schedule, 20, 43), 70);
    assert_eq!(compute(&schedule, 20, 48), 20);
    assert_eq!(compute(&schedule, 20, 49), 10);
    assert_eq!(compute(&schedule, 20, 50), 0);
    assert_eq!(compute(&schedule, 20, 51), 0);
    assert_eq!(compute(&schedule, 20, 52), 0);
    assert_eq!(compute(&schedule, 20, 0xff), 0);
}

#[test]
fn multi_linear_returns_time_now_before_the_starting_point_error() {
    let schedule = multi_linear_schedule([(3, 0, 0), (10, 10, 10), (100, 20, 10)]);

    assert_eq!(
        compute_result(&schedule, 20, 10),
        Err(TIME_NOW_BEFORE_THE_STARTING_POINT_ERROR)
    );
}

#[test]
fn multi_linear_starting_point_check() {
    let schedule = multi_linear_schedule([(3, 0, 0), (20, 20, 20), (200, 40, 20)]);

    let assert_all =
        |schedule: &MultiLinearScheduleOf<Test>, duration_since_start: u8, value: u8| {
            for starting_point in [0, 10, 20, 0xff] {
                let now = match starting_point.checked_add(&duration_since_start) {
                    None => continue,
                    Some(val) => val,
                };

                assert_eq!(
                    compute(schedule, starting_point, now),
                    value,
                    "{} {}",
                    duration_since_start,
                    value
                );
            }
        };

    assert_all(&schedule, 0, 220);
    assert_all(&schedule, 1, 220);
    assert_all(&schedule, 2, 220);
    assert_all(&schedule, 19, 220);
    assert_all(&schedule, 20, 220);
    assert_all(&schedule, 21, 219);
    assert_all(&schedule, 22, 218);
    assert_all(&schedule, 38, 202);
    assert_all(&schedule, 39, 201);
    assert_all(&schedule, 40, 200);
    assert_all(&schedule, 41, 190);
    assert_all(&schedule, 42, 180);
    assert_all(&schedule, 43, 170);
    assert_all(&schedule, 58, 20);
    assert_all(&schedule, 59, 10);
    assert_all(&schedule, 60, 0);
    assert_all(&schedule, 61, 0);
    assert_all(&schedule, 62, 0);
    assert_all(&schedule, 0xff, 0);
}
