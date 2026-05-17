use macros::process_str;

#[test]
fn string_unchanged() {
	const V: &str = process_str!("aaa");

	assert_eq!("aaa", V);
}

mod remove {
	use super::*;

	#[test]
	fn remove_parts() {
		const V: &str = process_str!("abcd", remove("a", "bc"));

		assert_eq!("d", V);
	}
}

mod lowercase {
	use super::*;

	#[test]
	fn make_lowercase() {
		const V: &str = process_str!("AAA", to_lowercase);

		assert_eq!("aaa", V);
	}

	#[test]
	fn apply_in_order() {
		const V: &str = process_str!(
			"ABC.123A",
			to_lowercase,
			remove("a", "B"),
			trim_numeric_suffix("."),
		);

		assert_eq!("bc", V);
	}
}

mod trim_numeric_suffix {
	use super::*;

	#[test]
	fn trim_suffix() {
		const V: &str = process_str!("aaa.123", trim_numeric_suffix("."));

		assert_eq!("aaa", V);
	}

	#[test]
	fn trim_only_suffix() {
		const V: &str = process_str!("aaa.123.123", trim_numeric_suffix("."));

		assert_eq!("aaa.123", V);
	}

	#[test]
	fn do_not_trim_if_delimiter_differs() {
		const V: &str = process_str!("aaa.123", trim_numeric_suffix("_"));

		assert_eq!("aaa.123", V);
	}

	#[test]
	fn do_not_trim_if_suffix_not_fully_numeric() {
		const V: &str = process_str!("aaa.123a", trim_numeric_suffix("."));

		assert_eq!("aaa.123a", V);
	}

	#[test]
	fn do_not_trim_if_whole_string_is_the_suffix() {
		const V: &str = process_str!(".123", trim_numeric_suffix("."));

		assert_eq!(".123", V);
	}
}
