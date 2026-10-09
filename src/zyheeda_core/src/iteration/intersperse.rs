use std::iter::{Fuse, FusedIterator};

pub trait IntersperseItem<T>: Iterator<Item = T> + Sized
where
	T: Copy,
{
	fn intersperse_item(self, item: T) -> IterInterspersed<Self, T>;
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

pub trait IntersperseWithItem<T>: Iterator<Item = T> + Sized {
	fn intersperse_with_item<F>(self, with_item: F) -> IterInterspersedWith<Self, F, T>
	where
		F: FnMut() -> T;
}

impl<T> IntersperseWithItem<T::Item> for T
where
	T: Iterator,
{
	fn intersperse_with_item<F>(self, with_item: F) -> IterInterspersedWith<Self, F, T::Item>
	where
		F: FnMut() -> T::Item,
	{
		IterInterspersedWith {
			iter: self.fuse(),
			intersperse: with_item,
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
		match self.mode.advance(&mut self.iter)? {
			Mode::Start => self.iter.next(),
			Mode::YieldItem(item) => Some(item),
			Mode::Intersperse => Some(self.intersperse),
		}
	}
}

impl<TIter, T> FusedIterator for IterInterspersed<TIter, T>
where
	TIter: Iterator<Item = T>,
	T: Copy,
{
}

pub struct IterInterspersedWith<TIter, F, T> {
	iter: Fuse<TIter>,
	intersperse: F,
	mode: Mode<T>,
}

impl<TIter, F, T> Iterator for IterInterspersedWith<TIter, F, T>
where
	TIter: Iterator<Item = T>,
	F: FnMut() -> T,
{
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		match self.mode.advance(&mut self.iter)? {
			Mode::Start => self.iter.next(),
			Mode::YieldItem(item) => Some(item),
			Mode::Intersperse => Some((self.intersperse)()),
		}
	}
}

impl<TIter, F, T> FusedIterator for IterInterspersedWith<TIter, F, T>
where
	TIter: Iterator<Item = T>,
	F: FnMut() -> T,
{
}

enum Mode<T> {
	Start,
	YieldItem(T),
	Intersperse,
}

impl<T> Mode<T> {
	fn advance(&mut self, iter: &mut impl Iterator<Item = T>) -> Option<Self> {
		let mut mode = match self {
			Mode::Start => Mode::Intersperse,
			Mode::YieldItem(_) => Mode::Intersperse,
			Mode::Intersperse => Mode::YieldItem(iter.next()?),
		};

		std::mem::swap(&mut mode, self);

		Some(mode)
	}
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
	fn intersperse_is_fused() {
		let mut it = _NonFused::New.intersperse_item(",").take(6);

		assert_eq!((None, None), (it.next(), it.next()));
	}

	#[test]
	fn intersperse_with() {
		let called = &mut 0;
		let v = ["a", "b", "c"];

		let it = v
			.into_iter()
			.intersperse_with_item(|| {
				*called += 1;
				","
			})
			.take(6);

		assert_eq!(
			(vec!["a", ",", "b", ",", "c"], 2),
			(it.collect::<Vec<_>>(), *called),
		);
	}

	#[test]
	fn intersperse_with_is_fused() {
		let called = &mut false;

		let mut it = _NonFused::New
			.intersperse_with_item(|| {
				*called = true;
				","
			})
			.take(6);

		assert_eq!((None, None, false), (it.next(), it.next(), *called));
	}
}
