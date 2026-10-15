use crate::collections::len::Len;
use std::collections::HashSet;

impl<T> Len for HashSet<T> {
	fn len(&self) -> usize {
		self.len()
	}
}
