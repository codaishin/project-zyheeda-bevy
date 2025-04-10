pub mod array;
pub mod slice;
pub mod vec;

pub trait Iterate<'a>: 'a
where
	Self::TIter: Iterator<Item = Self::TItem>,
{
	type TItem;
	type TIter;

	fn iterate(&'a self) -> Self::TIter;
}
