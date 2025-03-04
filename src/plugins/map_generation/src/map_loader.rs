use bevy::asset::{Asset, AssetLoader, LoadContext, io::Reader};
use std::{
	error::Error,
	fmt::{Display, Formatter, Result as FmtResult},
	io::Error as IOError,
	marker::PhantomData,
	str::{Utf8Error, from_utf8},
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

	async fn load(
		&self,
		reader: &mut dyn Reader,
		_: &Self::Settings,
		_: &mut LoadContext<'_>,
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
