use super::UnitsPerSecond;

#[test]
fn set_value() {
	let speed = UnitsPerSecond::new(42.);

	assert_eq!(42., speed.unpack());
}

#[test]
fn min_zero() {
	let speed = UnitsPerSecond::new(-42.);

	assert_eq!(0., speed.unpack());
}
