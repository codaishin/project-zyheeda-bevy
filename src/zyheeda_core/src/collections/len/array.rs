use super::Len;

impl<const N: usize, T> Len for [T; N] {
	fn len(&self) -> usize {
		const { N }
	}
}
