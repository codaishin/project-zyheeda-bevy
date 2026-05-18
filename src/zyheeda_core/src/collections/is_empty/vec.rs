use super::IsEmpty;

impl<T> IsEmpty for Vec<T> {
	fn is_empty(&self) -> bool {
		self.is_empty()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn is_empty() {
		assert!(IsEmpty::is_empty(&vec![] as &Vec<i32>));
	}

	#[test]
	fn is_not_empty() {
		assert!(!IsEmpty::is_empty(&vec![11]));
	}
}
