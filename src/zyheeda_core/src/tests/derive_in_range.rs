use crate::errors::{Limit, NotInRange};
use macros::{InRange, new_valid};
use std::{fmt::Debug, ops::Deref};
use test_case::test_case;

#[derive(Debug, PartialEq, InRange)]
#[in_range(low = > 10, high = < 100)]
struct _Exclusive10To100(i32);

#[derive(Debug, PartialEq, InRange)]
#[in_range(low = 10, high = 100)]
struct _Inclusive10To100(i32);

#[test_case(_Exclusive10To100::try_new; "exclusive try_new")]
#[test_case(_Exclusive10To100::try_from; "exclusive try_from")]
#[test_case(_Inclusive10To100::try_new; "inclusive try_new")]
#[test_case(_Inclusive10To100::try_from; "inclusive try_from")]
fn try_from_ok<T>(cstr: fn(i32) -> Result<T, NotInRange<i32>>)
where
	T: Deref<Target = i32>,
{
	assert_eq!(Ok(42), cstr(42).map(|v| *v));
}

#[test_case(_Exclusive10To100::try_new; "try_new")]
#[test_case(_Exclusive10To100::try_from; "try_from")]
fn too_small_error_exclusive(cstr: fn(i32) -> Result<_Exclusive10To100, NotInRange<i32>>) {
	assert_eq!(
		[
			Err(NotInRange {
				value: 9,
				lower_limit: Limit::Exclusive(10),
				upper_limit: Limit::Exclusive(100),
			}),
			Err(NotInRange {
				value: 10,
				lower_limit: Limit::Exclusive(10),
				upper_limit: Limit::Exclusive(100),
			}),
			Ok(_Exclusive10To100(11)),
		],
		[cstr(9), cstr(10), cstr(11)]
	);
}

#[test_case(_Inclusive10To100::try_new; "try_new")]
#[test_case(_Inclusive10To100::try_from; "try_from")]
fn too_small_error_inclusive(cstr: fn(i32) -> Result<_Inclusive10To100, NotInRange<i32>>) {
	assert_eq!(
		[
			Err(NotInRange {
				value: 9,
				lower_limit: Limit::Inclusive(10),
				upper_limit: Limit::Inclusive(100),
			}),
			Ok(_Inclusive10To100(10)),
			Ok(_Inclusive10To100(11)),
		],
		[cstr(9), cstr(10), cstr(11)]
	);
}

#[test_case(_Exclusive10To100::try_new; "try_new")]
#[test_case(_Exclusive10To100::try_from; "try_from")]
fn too_large_error_exclusive(cstr: fn(i32) -> Result<_Exclusive10To100, NotInRange<i32>>) {
	assert_eq!(
		[
			Ok(_Exclusive10To100(99)),
			Err(NotInRange {
				value: 100,
				lower_limit: Limit::Exclusive(10),
				upper_limit: Limit::Exclusive(100),
			}),
			Err(NotInRange {
				value: 101,
				lower_limit: Limit::Exclusive(10),
				upper_limit: Limit::Exclusive(100),
			}),
		],
		[cstr(99), cstr(100), cstr(101)]
	);
}

#[test_case(_Inclusive10To100::try_new; "try_new")]
#[test_case(_Inclusive10To100::try_from; "try_from")]
fn too_large_error_inclusive(cstr: fn(i32) -> Result<_Inclusive10To100, NotInRange<i32>>) {
	assert_eq!(
		[
			Ok(_Inclusive10To100(99)),
			Ok(_Inclusive10To100(100)),
			Err(NotInRange {
				value: 101,
				lower_limit: Limit::Inclusive(10),
				upper_limit: Limit::Inclusive(100),
			}),
		],
		[cstr(99), cstr(100), cstr(101)]
	);
}

#[test_case(_Exclusive10To100::try_new; "exclusive try_new")]
#[test_case(_Exclusive10To100::try_from; "exclusive try_from")]
#[test_case(_Inclusive10To100::try_new; "inclusive try_new")]
#[test_case(_Inclusive10To100::try_from; "inclusive try_from")]

fn deref<T>(cstr: fn(i32) -> Result<T, NotInRange<i32>>) -> Result<(), NotInRange<i32>>
where
	T: Deref<Target = i32>,
{
	let value = cstr(42)?;

	assert_eq!(42, *value);
	Ok(())
}

#[test_case(_Exclusive10To100::try_new; "try_new")]
#[test_case(_Exclusive10To100::try_from; "try_from")]
fn unwrap_exclusive(
	cstr: fn(i32) -> Result<_Exclusive10To100, NotInRange<i32>>,
) -> Result<(), NotInRange<i32>> {
	let value = cstr(42)?;

	assert_eq!(42, value.unwrap());
	Ok(())
}

#[test_case(_Inclusive10To100::try_new; "try_new")]
#[test_case(_Inclusive10To100::try_from; "try_from")]
fn unwrap_inclusive(
	cstr: fn(i32) -> Result<_Inclusive10To100, NotInRange<i32>>,
) -> Result<(), NotInRange<i32>> {
	let value = cstr(42)?;

	assert_eq!(42, value.unwrap());
	Ok(())
}

#[test]
fn too_small_display() {
	let error = NotInRange {
		value: 10,
		lower_limit: Limit::Exclusive(12),
		upper_limit: Limit::Inclusive(19),
	};

	assert_eq!(
		"Value `10` is out of bounds. Expected to be within `Exclusive(12)` and `Inclusive(19)`.",
		error.to_string()
	);
}

#[test]
fn new_valid_ok() {
	assert_eq!(_Exclusive10To100(12), new_valid!(_Exclusive10To100, 12));
}
