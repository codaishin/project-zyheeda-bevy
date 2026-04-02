#[macro_export]
macro_rules! all {
	($method:ident( $first:expr, $($rest:expr),+ $(,)? )) => {
		$first.$method() $(&& $rest.$method())+
	};
}

pub use all;

#[cfg(test)]
mod tests {
	#[test]
	fn all_true() {
		assert!(all!(is_positive(1i32, 2i32, 3i32)))
	}

	#[test]
	fn none_true() {
		assert!(!all!(is_positive(-1i32, -2i32, -3i32)))
	}

	#[test]
	fn some_true() {
		assert!(!all!(is_positive(-1i32, 2i32, -3i32)))
	}
}
