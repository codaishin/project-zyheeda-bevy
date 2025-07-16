use bevy::prelude::*;
use std::marker::PhantomData;

#[derive(Component, Debug)]
#[component(immutable)]
pub(crate) struct MapImage<TCell, TImage = Image>
where
	TImage: Asset,
{
	pub(crate) image: Handle<TImage>,
	_c: PhantomData<TCell>,
}

impl<TCell, TImage> From<Handle<TImage>> for MapImage<TCell, TImage>
where
	TImage: Asset,
{
	fn from(image: Handle<TImage>) -> Self {
		Self {
			image,
			_c: PhantomData,
		}
	}
}
impl<TCell, TImage> PartialEq for MapImage<TCell, TImage>
where
	TImage: Asset,
{
	fn eq(&self, other: &Self) -> bool {
		self.image == other.image
	}
}
