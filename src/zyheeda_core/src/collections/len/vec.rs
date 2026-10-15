use super::Len;

impl<T> Len for Vec<T> {
	fn len(&self) -> usize {
		self.len()
	}
}
