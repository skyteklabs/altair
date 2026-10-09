mod common;

use common::*;
use instance_licence::{InstanceLicenceReason, LicenceStatus, Notice, evaluate};
use licence::*;
use time::Date;
use time::macros::date;

/// The fixture Licence expires at the end of this day.
const EXPIRES: Date = date!(2027 - 10 - 08);

fn licence_expiring(expires: Date) -> Licence {
    Licence {
        expires,
        ..licence_for(THIS, "key-a")
    }
}

fn evaluate_on(licence: &Licence, today: Date) -> LicenceStatus {
    evaluate(
        Some(&file_signed_by(&licence.payload_bytes(), 7)),
        InstanceId::parse(THIS).unwrap(),
        &trusted(&[("key-a", 7)]),
        today,
    )
}

fn status_on(today: Date) -> LicenceStatus {
    evaluate_on(&licence_expiring(EXPIRES), today)
}

#[test]
fn the_day_before_expiry_is_valid() {
    assert!(matches!(
        status_on(date!(2027 - 10 - 07)),
        LicenceStatus::Valid(_)
    ));
}

#[test]
fn the_expiry_date_itself_is_valid() {
    assert!(matches!(status_on(EXPIRES), LicenceStatus::Valid(_)));
}

#[test]
fn the_first_day_after_expiry_is_the_grace_period() {
    assert_eq!(
        status_on(date!(2027 - 10 - 09)),
        LicenceStatus::GracePeriod(licence_expiring(EXPIRES))
    );
}

#[test]
fn the_14th_day_after_expiry_is_the_grace_period() {
    assert!(matches!(
        status_on(date!(2027 - 10 - 22)),
        LicenceStatus::GracePeriod(_)
    ));
}

#[test]
fn the_15th_day_after_expiry_is_expired() {
    assert_eq!(
        status_on(date!(2027 - 10 - 23)),
        LicenceStatus::Expired(licence_expiring(EXPIRES))
    );
}

#[test]
fn a_licence_for_another_instance_is_invalid_even_past_its_grace_period() {
    let other = Licence {
        instance_id: InstanceId::parse(OTHER).unwrap(),
        ..licence_expiring(EXPIRES)
    };
    assert_eq!(
        evaluate_on(&other, date!(2028 - 01 - 01)),
        LicenceStatus::Invalid(InstanceLicenceReason::WrongInstanceId)
    );
}

#[test]
fn valid_and_grace_period_may_write() {
    assert!(status_on(EXPIRES).may_write());
    assert!(status_on(date!(2027 - 10 - 22)).may_write());
}

#[test]
fn expired_may_not_write() {
    assert!(!status_on(date!(2027 - 10 - 23)).may_write());
}

#[test]
fn no_invalid_reason_may_write() {
    use InstanceLicenceReason::*;
    for reason in [
        NoLicence,
        Malformed,
        UnsupportedFormatVersion,
        UnknownKeyId,
        BadSignature,
        WrongInstanceId,
    ] {
        assert!(!LicenceStatus::Invalid(reason).may_write(), "{reason:?}");
    }
}

fn notice_on(today: Date) -> Option<Notice> {
    status_on(today).notice_due(today)
}

#[test]
fn expiry_warnings_are_due_30_14_and_7_days_before_expiry() {
    assert_eq!(
        notice_on(date!(2027 - 09 - 08)),
        Some(Notice::ExpiryWarning { days_left: 30 })
    );
    assert_eq!(
        notice_on(date!(2027 - 09 - 24)),
        Some(Notice::ExpiryWarning { days_left: 14 })
    );
    assert_eq!(
        notice_on(date!(2027 - 10 - 01)),
        Some(Notice::ExpiryWarning { days_left: 7 })
    );
}

#[test]
fn no_expiry_warning_on_the_days_around_each_warning() {
    for today in [
        date!(2027 - 09 - 07),
        date!(2027 - 09 - 09),
        date!(2027 - 09 - 23),
        date!(2027 - 09 - 25),
        date!(2027 - 09 - 30),
        date!(2027 - 10 - 02),
        EXPIRES,
    ] {
        assert_eq!(notice_on(today), None, "{today}");
    }
}

#[test]
fn the_grace_period_banner_is_due_on_every_grace_period_day() {
    let mut today = date!(2027 - 10 - 09);
    for _ in 0..14 {
        assert_eq!(notice_on(today), Some(Notice::GracePeriodBanner), "{today}");
        today = today.next_day().unwrap();
    }
}

#[test]
fn the_grace_period_banner_is_not_due_while_valid() {
    assert_eq!(notice_on(EXPIRES), None);
    assert_eq!(notice_on(date!(2027 - 10 - 07)), None);
}

#[test]
fn the_grace_period_banner_is_not_due_once_expired() {
    assert_eq!(notice_on(date!(2027 - 10 - 23)), None);
    assert_eq!(
        LicenceStatus::Invalid(InstanceLicenceReason::NoLicence).notice_due(EXPIRES),
        None
    );
}

#[test]
fn a_renewed_licence_brings_an_expired_instance_straight_back_to_valid() {
    let today = date!(2027 - 11 - 01);
    assert!(matches!(status_on(today), LicenceStatus::Expired(_)));

    let renewed = evaluate_on(&licence_expiring(date!(2028 - 10 - 08)), today);
    assert!(matches!(renewed, LicenceStatus::Valid(_)));
    assert!(renewed.may_write());
}

#[test]
fn a_valid_licence_brings_an_instance_with_no_licence_straight_back_to_valid() {
    let today = date!(2027 - 11 - 01);
    let none = evaluate(
        None,
        InstanceId::parse(THIS).unwrap(),
        &trusted(&[("key-a", 7)]),
        today,
    );
    assert!(!none.may_write());

    let installed = evaluate_on(&licence_expiring(date!(2028 - 10 - 08)), today);
    assert!(matches!(installed, LicenceStatus::Valid(_)));
    assert!(installed.may_write());
}
