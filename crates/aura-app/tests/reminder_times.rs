//! Times as models actually write them (042 round 2). São Paulo = UTC-3.
use aura_app::reminders::parse_time;

const SAO_PAULO: i32 = -3 * 3600;
// 2026-10-14 09:00 in São Paulo = 12:00 UTC.
const NINE_AM: i64 = 1_791_979_200;

#[test]
fn plain_local_time_is_sao_paulo() {
    assert_eq!(parse_time("2026-10-14T09:00", SAO_PAULO).unwrap(), NINE_AM);
    assert_eq!(
        parse_time("2026-10-14T09:00:00", SAO_PAULO).unwrap(),
        NINE_AM
    );
}

#[test]
fn fractional_seconds_are_accepted() {
    assert_eq!(
        parse_time("2026-10-14T09:00:00.000", SAO_PAULO).unwrap(),
        NINE_AM
    );
}

#[test]
fn an_offset_without_seconds_is_accepted() {
    assert_eq!(
        parse_time("2026-10-14T09:00-03:00", SAO_PAULO).unwrap(),
        NINE_AM
    );
    assert_eq!(parse_time("2026-10-14T12:00Z", SAO_PAULO).unwrap(), NINE_AM);
}
