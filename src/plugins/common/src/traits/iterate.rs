pub mod array;
pub mod slice;
pub mod vec;

pub trait Iterate {
	type TItem<'a>
	where
		Self: 'a;

	fn iterate(&self) -> impl Iterator<Item = Self::TItem<'_>>;
}
