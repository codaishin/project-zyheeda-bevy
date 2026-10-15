use crate::collections::is_empty::IsEmpty;
use std::collections::HashSet;

impl<T> IsEmpty for HashSet<T> {
	fn is_empty(&self) -> bool {
		self.is_empty()
	}
}
