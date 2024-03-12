use bevy::{
	asset::{io::Reader, Asset, AssetLoader, AsyncReadExt, LoadContext},
	reflect::TypePath,
	utils::BoxedFuture,
};
use std::{
	error::Error,
	fmt::{Display, Formatter, Result as FmtResult},
	io::Error as IOError,
	str::{from_utf8, Utf8Error},
};

#[derive(TypePath, Asset)]
pub struct Map(pub String);

pub struct MapLoader;

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

impl AssetLoader for MapLoader {
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
				Ok(Ok(str)) => Ok(Map(str.to_owned())),
			}
		})
	}
}
