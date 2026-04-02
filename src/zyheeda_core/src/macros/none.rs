#[macro_export]
macro_rules! none {
	($method:ident( $first:expr, $($rest:expr),+ $(,)? )) => {
		!$first.$method() $(&& !$rest.$method())+
	};
}

pub use none;

#[cfg(test)]
mod tests {
	#[test]
	fn false_because_all_return_true() {
		assert!(!none!(is_positive(1i32, 2i32, 3i32)))
	}

	#[test]
	fn true_because_all_return_false() {
		assert!(none!(is_positive(-1i32, -2i32, -3i32)))
	}

	#[test]
	fn false_because_some_return_true() {
		assert!(!none!(is_positive(-1i32, 2i32, -3i32)))
	}
}
