use crate::collections::len::Len;

impl<T> Len for [T] {
	fn len(&self) -> usize {
		self.len()
	}
}
