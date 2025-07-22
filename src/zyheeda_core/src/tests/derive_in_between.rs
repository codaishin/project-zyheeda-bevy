use crate::errors::NotInBetween;
use macros::{InBetween, new_valid};
use std::fmt::Debug;
use test_case::test_case;

#[derive(Debug, PartialEq, InBetween)]
#[in_between(low = 10, high = 100)]
struct _10To100(i32);

#[test_case(_10To100::try_new; "try_new")]
#[test_case(_10To100::try_from; "try_from")]
fn try_from_ok(cstr: fn(i32) -> Result<_10To100, NotInBetween<i32>>) {
	assert_eq!(Ok(_10To100(42)), cstr(42));
}

#[test_case(_10To100::try_new; "try_new")]
#[test_case(_10To100::try_from; "try_from")]
fn too_small_error<T>(cstr: fn(i32) -> Result<T, NotInBetween<i32>>)
where
	T: Debug + PartialEq,
{
	assert_eq!(
		Err(NotInBetween {
			value: 10,
			lower_limit: 10,
			upper_limit: 100,
		}),
		cstr(10)
	);
}

#[test_case(_10To100::try_new; "try_new")]
#[test_case(_10To100::try_from; "try_from")]
fn too_large_error<T>(cstr: fn(i32) -> Result<T, NotInBetween<i32>>)
where
	T: Debug + PartialEq,
{
	assert_eq!(
		Err(NotInBetween {
			value: 100,
			lower_limit: 10,
			upper_limit: 100,
		}),
		cstr(100)
	);
}

#[test_case(_10To100::try_new; "try_new")]
#[test_case(_10To100::try_from; "try_from")]
fn deref(cstr: fn(i32) -> Result<_10To100, NotInBetween<i32>>) -> Result<(), NotInBetween<i32>> {
	let value = cstr(42)?;

	assert_eq!(42, *value);
	Ok(())
}

#[test_case(_10To100::try_new; "try_new")]
#[test_case(_10To100::try_from; "try_from")]
fn unwrap(cstr: fn(i32) -> Result<_10To100, NotInBetween<i32>>) -> Result<(), NotInBetween<i32>> {
	let value = cstr(42)?;

	assert_eq!(42, value.unwrap());
	Ok(())
}

#[test]
fn too_small_display() {
	let error = NotInBetween {
		value: 10,
		lower_limit: 12,
		upper_limit: 19,
	};

	assert_eq!(
		"Value `10` is out of bounds. Expected to be greater than `12` and lesser than `19`.",
		error.to_string()
	);
}

#[test]
fn static_min() {
	const MIN: _10To100 = new_valid!(_10To100, 12);

	assert_eq!(_10To100(12), MIN);
}
