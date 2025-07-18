use bevy::prelude::*;
use std::marker::PhantomData;

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct MapColorLookup<TCell> {
	pub(crate) floor: Color,
	_c: PhantomData<TCell>,
}

impl<TCell> MapColorLookup<TCell> {
	pub(crate) fn new(floor: Color) -> Self {
		Self {
			floor,
			_c: PhantomData,
		}
	}
}

impl<TCell> Clone for MapColorLookup<TCell> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<TCell> Copy for MapColorLookup<TCell> {}

#[derive(Resource, Debug)]
pub(crate) struct MapColorLookupImage<TCell, TImage = Image>
where
	TImage: Asset,
{
	pub(crate) floor: Handle<TImage>,
	_c: PhantomData<TCell>,
}

impl<TCell, TImage> MapColorLookupImage<TCell, TImage>
where
	TImage: Asset,
{
	pub(crate) fn new(floor: Handle<TImage>) -> Self {
		Self {
			floor,
			_c: PhantomData,
		}
	}
}

impl<TCell, TImage> PartialEq for MapColorLookupImage<TCell, TImage>
where
	TImage: Asset,
{
	fn eq(&self, other: &Self) -> bool {
		self.floor == other.floor
	}
}
