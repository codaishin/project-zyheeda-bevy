use crate::traits::is_empty::IsEmpty;

impl<const N: usize, T> IsEmpty for [T; N] {
	fn is_empty(&self) -> bool {
		N == 0
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn is_empty() {
		assert!(IsEmpty::is_empty(&[] as &[i32; 0]));
	}

	#[test]
	fn is_not_empty() {
		assert!(!IsEmpty::is_empty(&[11] as &[i32; 1]));
	}
}
