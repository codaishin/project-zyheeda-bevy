use bevy::asset::{Asset, AssetLoader, LoadContext, io::Reader};
use common::traits::thread_safe::ThreadSafe;
use std::{
	error::Error,
	fmt::{Debug, Display, Formatter, Result as FmtResult},
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
pub enum TextLoaderError<TError> {
	IO(IOError),
	Parse(Utf8Error),
	Custom(TError),
}

impl<TError> TextLoaderError<TError> {
	fn custom(error: TError) -> Self {
		Self::Custom(error)
	}
}

impl<TError> Display for TextLoaderError<TError>
where
	TError: Display,
{
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		match self {
			TextLoaderError::IO(error) => write!(f, "IO: {error}"),
			TextLoaderError::Parse(error) => write!(f, "Parse: {error}"),
			TextLoaderError::Custom(error) => write!(f, "Map Size: {error}"),
		}
	}
}

impl<TError> Error for TextLoaderError<TError> where TError: Debug + Display {}

impl<TAsset> AssetLoader for TextLoader<TAsset>
where
	TAsset: TryFrom<String> + Asset,
	TAsset::Error: Debug + Display + ThreadSafe,
{
	type Asset = TAsset;
	type Settings = ();
	type Error = TextLoaderError<TAsset::Error>;

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
			Ok(Ok(str)) => TAsset::try_from(str.to_string()).map_err(TextLoaderError::custom),
		}
	}
}
