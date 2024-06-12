pub struct Iter<TValue>(pub Option<TValue>);

pub trait IterFinite
where
	Self: Sized,
{
	fn iterator() -> Iter<Self>;
	fn next(current: &Iter<Self>) -> Option<Self>;
}

impl<TIterKey: IterFinite + Copy> Iterator for Iter<TIterKey> {
	type Item = TIterKey;

	fn next(&mut self) -> Option<Self::Item> {
		let current = &self.0?;
		self.0 = TIterKey::next(self);

		Some(*current)
	}
}

pub struct Infinite<TValue>(pub TValue);

impl<TValue: IterInfinite + Copy> Infinite<TValue> {
	pub fn next_infinite(&mut self) -> TValue {
		let current = self.0;
		self.0 = TValue::next_infinite(self);

		current
	}
}

pub trait IterInfinite
where
	Self: Sized,
{
	fn iterator_infinite() -> Infinite<Self>;
	fn next_infinite(current: &Infinite<Self>) -> Self;
}

impl<TItem: IterInfinite + Copy> Iterator for Infinite<TItem> {
	type Item = TItem;

	fn next(&mut self) -> Option<Self::Item> {
		Some(self.next_infinite())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Clone, Copy, PartialEq, Debug)]
	struct _MyType(usize);

	impl IterFinite for _MyType {
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

	impl IterInfinite for _MyType {
		fn iterator_infinite() -> Infinite<Self> {
			Infinite(_MyType(0))
		}

		fn next_infinite(current: &Infinite<Self>) -> Self {
			_MyType(current.0 .0 + 1)
		}
	}

	#[test]
	fn iterate_keys_infinite() {
		assert_eq!(
			(0..100).map(_MyType).collect::<Vec<_>>(),
			_MyType::iterator_infinite().take(100).collect::<Vec<_>>()
		);
	}
}
