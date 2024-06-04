pub mod array;
pub mod slice;
pub mod vec;

pub trait Iterate<TItem> {
	fn iterate<'a>(&'a self) -> impl DoubleEndedIterator<Item = &'a TItem>
	where
		TItem: 'a;
}
