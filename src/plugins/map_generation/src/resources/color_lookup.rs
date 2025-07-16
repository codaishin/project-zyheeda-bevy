use bevy::prelude::*;
use std::marker::PhantomData;

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct ColorLookup<TCell> {
	pub(crate) floor: Color,
	_c: PhantomData<TCell>,
}

impl<TCell> ColorLookup<TCell> {
	pub(crate) fn new(floor: Color) -> Self {
		Self {
			floor,
			_c: PhantomData,
		}
	}
}

impl<TCell> Clone for ColorLookup<TCell> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<TCell> Copy for ColorLookup<TCell> {}

#[derive(Resource, Debug)]
pub(crate) struct ColorLookupImage<TCell, TImage = Image>
where
	TImage: Asset,
{
	pub(crate) floor: Handle<TImage>,
	_c: PhantomData<TCell>,
}

impl<TCell, TImage> ColorLookupImage<TCell, TImage>
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

impl<TCell, TImage> PartialEq for ColorLookupImage<TCell, TImage>
where
	TImage: Asset,
{
	fn eq(&self, other: &Self) -> bool {
		self.floor == other.floor
	}
}
