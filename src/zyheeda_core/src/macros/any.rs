#[macro_export]
macro_rules! any {
	($method:ident( $first:expr, $($rest:expr),+ $(,)? )) => {
		$first.$method() $(|| $rest.$method())+
	};
}

pub use any;

#[cfg(test)]
mod tests {
	#[test]
	fn all_true() {
		assert!(any!(is_positive(1i32, 2i32, 3i32)))
	}

	#[test]
	fn none_true() {
		assert!(!any!(is_positive(-1i32, -2i32, -3i32)))
	}

	#[test]
	fn some_true() {
		assert!(any!(is_positive(-1i32, 2i32, -3i32)))
	}
}
