use crate::traits::is_empty::IsEmpty;

impl<T> IsEmpty for [T] {
	fn is_empty(&self) -> bool {
		self.is_empty()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn is_empty() {
		assert!(IsEmpty::is_empty(&[] as &[i32]));
	}

	#[test]
	fn is_not_empty() {
		assert!(!IsEmpty::is_empty(&[11] as &[i32]));
	}
}
