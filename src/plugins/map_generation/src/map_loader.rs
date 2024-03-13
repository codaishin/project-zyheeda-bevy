use crate::traits::StringToCells;
use bevy::{
	asset::{io::Reader, Asset, AssetLoader, AsyncReadExt, LoadContext},
	math::primitives::Direction3d,
	reflect::TypePath,
	utils::BoxedFuture,
};
use std::{
	error::Error,
	fmt::{Display, Formatter, Result as FmtResult},
	io::Error as IOError,
	marker::PhantomData,
	str::{from_utf8, Utf8Error},
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum Shape {
	Single,
	End,
	Straight,
	Cross2,
	Cross3,
	Cross4,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum Cell {
	Corridor(Direction3d, Shape),
	Empty,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Cells(pub Vec<Vec<Cell>>);

#[derive(TypePath, Asset)]
pub struct Map(pub Cells);

pub struct MapLoader<TParser> {
	phantom_date: PhantomData<TParser>,
}

impl<T> Default for MapLoader<T> {
	fn default() -> Self {
		Self {
			phantom_date: PhantomData,
		}
	}
}

#[derive(Debug)]
pub enum MapLoadError {
	IO(IOError),
	Parse(Utf8Error),
}

impl Display for MapLoadError {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		match self {
			MapLoadError::IO(error) => write!(f, "IO: {}", error),
			MapLoadError::Parse(error) => write!(f, "Parse: {}", error),
		}
	}
}

impl Error for MapLoadError {}

impl<TParser: StringToCells + Send + Sync + 'static> AssetLoader for MapLoader<TParser> {
	type Asset = Map;
	type Settings = ();
	type Error = MapLoadError;

	fn load<'a>(
		&'a self,
		reader: &'a mut Reader,
		_settings: &'a Self::Settings,
		_load_context: &'a mut LoadContext,
	) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
		Box::pin(async {
			let mut bytes = Vec::new();
			let result = reader
				.read_to_end(&mut bytes)
				.await
				.map(|_| from_utf8(&bytes));

			match result {
				Err(error) => Err(MapLoadError::IO(error)),
				Ok(Err(error)) => Err(MapLoadError::Parse(error)),
				Ok(Ok(str)) => Ok(Map(TParser::string_to_cells(str))),
			}
		})
	}
}
