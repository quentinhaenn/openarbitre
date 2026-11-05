//! Writes and batch validation (T-4). US-4 matrix, case 11; A-1, A-11.
//! Cases 12 and 13 (author outside the list, device ≠ stamp.a): not representable, deviation E-1 of the design.

mod support;

use arbitre_core::*;
use support::*;

fn valid() -> Write {
    let mut w = write(record("d"), Change::field("f", b"v"), stamp(10, 0, 1));
    w.seen.insert(kind("f"), stamp(9, 0, 2));
    w.seen.insert(TargetKind::Delete, stamp(5, 3, 1));
    w
}

fn rejected(w: Write) -> Result<(), BatchError> {
    validate(&[valid(), w])
}

#[test]
fn us4_case11_full_write_of_each_author() {
    let auteurs = [
        Author::CliUser,
        Author::UiUser,
        Author::Assistant,
        Author::System,
    ];
    let batch: Vec<Write> = auteurs
        .iter()
        .map(|&author| Write { author, ..valid() })
        .collect();
    assert_eq!(validate(&batch), Ok(()));
    let deletion = Write {
        change: Change::Delete,
        ..valid()
    };
    assert_eq!(validate(&[deletion]), Ok(()));
}

#[test]
fn empty_batch_is_valid() {
    assert_eq!(validate(&[]), Ok(()));
}

#[test]
fn target_and_device() {
    let w = valid();
    assert_eq!(
        w.target(),
        Target {
            record: record("d"),
            kind: kind("f")
        }
    );
    assert_eq!(w.device(), dev(1));
    let s = Write {
        change: Change::Delete,
        ..valid()
    };
    assert_eq!(
        s.target(),
        Target {
            record: record("d"),
            kind: TargetKind::Delete
        }
    );
}

#[test]
fn seen_equal_to_stamp_rejected() {
    let mut w = valid();
    w.seen.insert(kind("f"), w.stamp);
    assert_eq!(
        rejected(w),
        Err(BatchError {
            index: 1,
            reason: Reason::SeenNotBefore
        })
    );
}

#[test]
fn seen_greater_than_stamp_rejected() {
    let mut w = valid();
    w.seen.insert(kind("f"), stamp(11, 0, 0));
    assert_eq!(
        rejected(w),
        Err(BatchError {
            index: 1,
            reason: Reason::SeenNotBefore
        })
    );
}

#[test]
fn only_second_seen_entry_faulty() {
    let mut w = valid();
    // Delete < Field: the first entry (deletion) is valid, the second (field) is faulty.
    w.seen.insert(kind("f"), stamp(10, 0, 2));
    assert_eq!(
        rejected(w),
        Err(BatchError {
            index: 1,
            reason: Reason::SeenNotBefore
        })
    );
}

#[test]
fn extra_seen_on_another_field_accepted() {
    let mut w = valid();
    w.seen.insert(kind("autre"), stamp(1, 0, 3));
    assert_eq!(validate(&[w]), Ok(()));
}

#[test]
fn empty_identifiers_rejected() {
    let empties = [
        Write {
            record: RecordId {
                module: String::new(),
                ..record("d")
            },
            ..valid()
        },
        Write {
            record: RecordId {
                category: String::new(),
                ..record("d")
            },
            ..valid()
        },
        Write {
            record: record(""),
            ..valid()
        },
        Write {
            change: Change::field("", b"v"),
            ..valid()
        },
    ];
    for w in empties {
        assert_eq!(
            rejected(w),
            Err(BatchError {
                index: 1,
                reason: Reason::EmptyIdentifier
            })
        );
    }
}

#[test]
fn empty_value_accepted() {
    assert_eq!(
        validate(&[Write {
            change: Change::field("f", b""),
            ..valid()
        }]),
        Ok(())
    );
}

#[test]
fn stamp_l_max_rejected() {
    let w = Write {
        stamp: stamp(L_MAX, 0, 1),
        ..valid()
    };
    assert_eq!(
        rejected(w),
        Err(BatchError {
            index: 1,
            reason: Reason::StampOutOfBounds
        })
    );
}

#[test]
fn stamp_l_max_minus_one_accepted() {
    assert_eq!(
        validate(&[Write {
            stamp: stamp(L_MAX - 1, 0, 1),
            ..valid()
        }]),
        Ok(())
    );
}

#[test]
fn index_of_first_faulty() {
    let faulty = Write {
        record: record(""),
        ..valid()
    };
    let batch = [
        valid(),
        valid(),
        faulty.clone(),
        Write {
            stamp: stamp(L_MAX, 0, 1),
            ..valid()
        },
        faulty,
    ];
    assert_eq!(
        validate(&batch),
        Err(BatchError {
            index: 2,
            reason: Reason::EmptyIdentifier
        })
    );
}
