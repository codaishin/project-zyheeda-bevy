#[macro_export]
macro_rules! hash_map {
	($($key:expr => $value:expr),* $(,)?) => {
		std::collections::hash_map::HashMap::from([$(($key, $value)),*])
	};
}

pub use hash_map;

#[cfg(test)]
mod tests {
	use std::collections::hash_map::HashMap;

	#[test]
	fn build_empty() {
		assert_eq!(HashMap::<&str, i32>::from([]), hash_map! {},);
	}

	#[test]
	fn build_single() {
		assert_eq!(
			HashMap::from([("a", 42)]),
			hash_map! {
				"a" => 42
			},
		);
	}

	#[test]
	fn build_multiple() {
		assert_eq!(
			HashMap::from([("a", 42), ("b", 55)]),
			hash_map! {
				"a" => 42,
				"b" => 55,
			},
		);
	}
}
