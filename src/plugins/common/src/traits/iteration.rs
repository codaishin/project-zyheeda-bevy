pub trait IterKey
where
	Self: Sized,
{
	fn iterator() -> Iter<Self>;
	fn next(current: &Iter<Self>) -> Option<Self>;
}

pub struct Iter<TValue>(pub Option<TValue>);

impl<TIterKey: IterKey + Copy> Iterator for Iter<TIterKey> {
	type Item = TIterKey;

	fn next(&mut self) -> Option<Self::Item> {
		let current = &self.0?;
		self.0 = TIterKey::next(self);

		Some(*current)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Clone, Copy, PartialEq, Debug)]
	struct _MyType(usize);

	impl IterKey for _MyType {
		fn iterator() -> Iter<Self> {
			Iter(Some(_MyType(0)))
		}

		fn next(current: &Iter<Self>) -> Option<_MyType> {
			match &current.0?.0 {
				0 => Some(_MyType(1)),
				1 => Some(_MyType(200)),
				_ => None,
			}
		}
	}

	#[test]
	fn iterate_keys() {
		assert_eq!(
			vec![_MyType(0), _MyType(1), _MyType(200),],
			_MyType::iterator().collect::<Vec<_>>()
		);
	}
}
