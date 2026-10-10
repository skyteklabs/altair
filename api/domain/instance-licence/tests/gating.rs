mod common;

use common::*;
use instance_licence::{InstanceLicenceReason, LicenceStatus, SignInRefusal, evaluate};
use licence::*;
use time::macros::date;

/// The fixture Licence has an Active User cap of 50.
const CAP: u32 = 50;

fn evaluate_at(address: &str) -> LicenceStatus {
    evaluate(
        Some(&file_signed_by(
            &licence_for(THIS, "key-a").payload_bytes(),
            7,
        )),
        InstanceId::parse(THIS).unwrap(),
        &InstanceAddress::new(address).unwrap(),
        &trusted(&[("key-a", 7)]),
        date!(2026 - 10 - 08),
    )
}

fn valid() -> LicenceStatus {
    evaluate_at(ADDRESS)
}

fn expired() -> LicenceStatus {
    LicenceStatus::Expired(licence_for(THIS, "key-a"))
}

fn invalid() -> LicenceStatus {
    LicenceStatus::Invalid(InstanceLicenceReason::NoLicence)
}

#[test]
fn the_licences_instance_address_is_accepted() {
    assert!(matches!(valid(), LicenceStatus::Valid(_)));
}

#[test]
fn the_instance_address_is_compared_without_regard_to_case() {
    assert!(matches!(
        evaluate_at("Bank.Example"),
        LicenceStatus::Valid(_)
    ));
}

#[test]
fn any_other_instance_address_is_refused() {
    for address in [
        "other.example",
        "app.bank.example",
        "bank.example.org",
        "bank",
    ] {
        assert_eq!(
            evaluate_at(address),
            LicenceStatus::Invalid(InstanceLicenceReason::WrongInstanceAddress),
            "{address}"
        );
    }
}

#[test]
fn a_licence_for_another_instance_at_another_address_reports_the_instance_id() {
    let other = Licence {
        instance_address: InstanceAddress::new("other.example").unwrap(),
        ..licence_for(OTHER, "key-a")
    };
    let status = evaluate(
        Some(&file_signed_by(&other.payload_bytes(), 7)),
        InstanceId::parse(THIS).unwrap(),
        &this_address(),
        &trusted(&[("key-a", 7)]),
        date!(2026 - 10 - 08),
    );
    assert_eq!(
        status,
        LicenceStatus::Invalid(InstanceLicenceReason::WrongInstanceId)
    );
}

#[test]
fn another_instance_address_leaves_the_instance_read_only() {
    assert!(!evaluate_at("other.example").may_write());
}

#[test]
fn a_feature_in_the_licence_is_licensed() {
    assert!(valid().is_feature_licensed("investing"));
}

#[test]
fn a_feature_not_in_the_licence_is_not_licensed() {
    assert!(!valid().is_feature_licensed("shared-spaces"));
}

#[test]
fn an_expired_instance_licenses_no_feature() {
    assert!(!expired().is_feature_licensed("investing"));
}

#[test]
fn an_invalid_licence_licenses_no_feature() {
    assert!(!invalid().is_feature_licensed("investing"));
}

#[test]
fn below_the_cap_active_and_new_users_may_sign_in() {
    assert_eq!(valid().may_sign_in(CAP - 1, true), Ok(()));
    assert_eq!(valid().may_sign_in(CAP - 1, false), Ok(()));
    assert_eq!(valid().may_sign_in(0, false), Ok(()));
}

#[test]
fn at_the_cap_an_active_user_may_sign_in() {
    assert_eq!(valid().may_sign_in(CAP, true), Ok(()));
}

#[test]
fn at_the_cap_a_new_or_returning_user_is_refused() {
    assert_eq!(
        valid().may_sign_in(CAP, false),
        Err(SignInRefusal::ActiveUserCapReached)
    );
}

#[test]
fn above_the_cap_an_active_user_may_sign_in_and_anyone_else_is_refused() {
    assert_eq!(valid().may_sign_in(CAP + 5, true), Ok(()));
    assert_eq!(
        valid().may_sign_in(CAP + 5, false),
        Err(SignInRefusal::ActiveUserCapReached)
    );
}

#[test]
fn on_an_expired_instance_only_active_users_may_sign_in() {
    assert_eq!(expired().may_sign_in(CAP, true), Ok(()));
    assert_eq!(
        expired().may_sign_in(0, false),
        Err(SignInRefusal::ExpiredInstance)
    );
}

#[test]
fn with_an_invalid_licence_only_active_users_may_sign_in() {
    assert_eq!(invalid().may_sign_in(0, true), Ok(()));
    assert_eq!(
        invalid().may_sign_in(0, false),
        Err(SignInRefusal::ExpiredInstance)
    );
}

#[test]
fn the_cap_warning_is_due_at_90_percent_of_the_cap_and_above() {
    // 90% of 50 is 45.
    for active_users in [45, 49, CAP, CAP + 5] {
        assert!(valid().cap_warning_due(active_users), "{active_users}");
    }
}

#[test]
fn the_cap_warning_is_not_due_below_90_percent_of_the_cap() {
    for active_users in [0, 44] {
        assert!(!valid().cap_warning_due(active_users), "{active_users}");
    }
}

#[test]
fn the_cap_warning_rounds_90_percent_up() {
    let small_cap = LicenceStatus::Valid(Licence {
        active_user_cap: ActiveUserCap::new(3).unwrap(),
        ..licence_for(THIS, "key-a")
    });
    // 90% of 3 is 2.7, so the warning starts at 3.
    assert!(!small_cap.cap_warning_due(2));
    assert!(small_cap.cap_warning_due(3));
}

#[test]
fn the_cap_warning_is_not_due_with_an_invalid_licence() {
    assert!(!invalid().cap_warning_due(1_000));
}

#[test]
fn the_cap_warning_is_not_due_on_an_expired_instance() {
    assert!(!expired().cap_warning_due(CAP));
}
