use bevy::asset::{io::Reader, Asset, AssetLoader, AsyncReadExt, LoadContext};
use std::{
	error::Error,
	fmt::{Display, Formatter, Result as FmtResult},
	io::Error as IOError,
	marker::PhantomData,
	str::{from_utf8, Utf8Error},
};

pub struct TextLoader<TParser> {
	phantom_date: PhantomData<TParser>,
}

impl<T> Default for TextLoader<T> {
	fn default() -> Self {
		Self {
			phantom_date: PhantomData,
		}
	}
}

#[derive(Debug)]
pub enum TextLoaderError {
	IO(IOError),
	Parse(Utf8Error),
}

impl Display for TextLoaderError {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		match self {
			TextLoaderError::IO(error) => write!(f, "IO: {}", error),
			TextLoaderError::Parse(error) => write!(f, "Parse: {}", error),
		}
	}
}

impl Error for TextLoaderError {}

impl<TAsset: From<String> + Asset> AssetLoader for TextLoader<TAsset> {
	type Asset = TAsset;
	type Settings = ();
	type Error = TextLoaderError;

	async fn load<'a>(
		&'a self,
		reader: &'a mut Reader<'_>,
		_settings: &'a Self::Settings,
		_load_context: &'a mut LoadContext<'_>,
	) -> Result<Self::Asset, Self::Error> {
		let mut bytes = Vec::new();
		let result = reader
			.read_to_end(&mut bytes)
			.await
			.map(|_| from_utf8(&bytes));

		match result {
			Err(error) => Err(TextLoaderError::IO(error)),
			Ok(Err(error)) => Err(TextLoaderError::Parse(error)),
			Ok(Ok(str)) => Ok(TAsset::from(str.to_string())),
		}
	}
}
