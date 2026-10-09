use std::iter::{Fuse, FusedIterator};

pub trait IntersperseItem<T>: Iterator<Item = T> + Sized
where
	T: Copy,
{
	fn intersperse_item(self, value: T) -> IterInterspersed<Self, T>;
}

impl<T> IntersperseItem<T::Item> for T
where
	T: Iterator,
	T::Item: Copy,
{
	fn intersperse_item(self, item: T::Item) -> IterInterspersed<Self, T::Item> {
		IterInterspersed {
			iter: self.fuse(),
			intersperse: item,
			mode: Mode::Start,
		}
	}
}

pub struct IterInterspersed<TIter, T> {
	iter: Fuse<TIter>,
	intersperse: T,
	mode: Mode<T>,
}

impl<TIter, T> Iterator for IterInterspersed<TIter, T>
where
	TIter: Iterator<Item = T>,
	T: Copy,
{
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		match self.mode {
			Mode::Start => {
				self.mode = Mode::Intersperse;
				self.iter.next()
			}
			Mode::YieldItem(item) => {
				self.mode = Mode::Intersperse;
				Some(item)
			}
			Mode::Intersperse => {
				self.mode = Mode::YieldItem(self.iter.next()?);
				Some(self.intersperse)
			}
		}
	}
}

impl<TIter, T> FusedIterator for IterInterspersed<TIter, T>
where
	TIter: Iterator<Item = T>,
	T: Copy,
{
}

enum Mode<T> {
	Start,
	YieldItem(T),
	Intersperse,
}

#[cfg(test)]
mod tests {
	use super::*;

	enum _NonFused {
		New,
		Old,
	}

	impl Iterator for _NonFused {
		type Item = &'static str;

		fn next(&mut self) -> Option<Self::Item> {
			match self {
				_NonFused::Old => Some("a"),
				iter => {
					*iter = Self::Old;
					None
				}
			}
		}
	}

	#[test]
	fn intersperse() {
		let v = ["a", "b", "c"];

		let it = v.into_iter().intersperse_item(",").take(6);

		assert_eq!(vec!["a", ",", "b", ",", "c"], it.collect::<Vec<_>>());
	}

	#[test]
	fn is_fused() {
		let mut it = _NonFused::New.intersperse_item(",").take(6);

		assert_eq!((None, None), (it.next(), it.next()));
	}
}
