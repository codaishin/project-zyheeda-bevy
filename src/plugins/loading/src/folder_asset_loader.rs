use bevy::{
	asset::{Asset, AssetLoader, LoadContext, io::Reader},
	reflect::TypePath,
};
use common::traits::handles_custom_assets::{AssetFileExtensions, LoadFrom};
use serde::Deserialize;
use serde_json::error::Error as SerdeJsonError;
use std::{
	error::Error,
	fmt::{Display, Formatter, Result as FmtResult},
	io::Error as IOError,
	marker::PhantomData,
	str::{Utf8Error, from_utf8},
};

/// Generic asset loader that always returns `Ok`. Errors are stored within the `Ok`
/// side of the result, so we can handle them on an individual level and prevent
/// bevy from stopping the load process when encountering an error.
pub(crate) struct FolderAssetLoader<TAsset, TDto> {
	phantom_data: PhantomData<(TAsset, TDto)>,
}

impl<TAsset, TDto> FolderAssetLoader<TAsset, TDto> {
	async fn read<'a>(
		reader: &mut dyn Reader,
		buffer: &'a mut Vec<u8>,
	) -> Result<&'a str, ReadError> {
		reader.read_to_end(buffer).await.map_err(ReadError::IO)?;
		from_utf8(buffer).map_err(ReadError::ParseChars)
	}
}

impl<TAsset, TDto> Default for FolderAssetLoader<TAsset, TDto> {
	fn default() -> Self {
		Self {
			phantom_data: PhantomData,
		}
	}
}

impl<TAsset, TDto> AssetLoader for FolderAssetLoader<TAsset, TDto>
where
	TAsset: Asset + LoadFrom<TDto>,
	for<'a> TDto: Deserialize<'a> + AssetFileExtensions + Sync + Send + 'static,
{
	type Asset = LoadResult<TAsset>;
	type Settings = ();
	type Error = UnreachableError;

	fn extensions(&self) -> &[&str] {
		TDto::asset_file_extensions()
	}

	async fn load(
		&self,
		reader: &mut dyn Reader,
		_: &Self::Settings,
		context: &mut LoadContext<'_>,
	) -> Result<Self::Asset, Self::Error> {
		let buffer = &mut vec![];

		let dto = match Self::read(reader, buffer).await {
			Err(ReadError::IO(err)) => return LoadResult::io_error(err),
			Err(ReadError::ParseChars(err)) => return LoadResult::char_error(err),
			Ok(str) => serde_json::from_str(str),
		};

		match dto {
			Ok(dto) => LoadResult::ok(TAsset::load_from(dto, context)),
			Err(err) => LoadResult::parse_error(err),
		}
	}
}

#[derive(Asset, TypePath, Debug, PartialEq)]
pub(crate) enum LoadResult<TAsset: Asset, TError: Sync + Send + TypePath + 'static = LoadError> {
	Ok(TAsset),
	Err(TError),
}

impl<TAsset: Asset> LoadResult<TAsset, LoadError> {
	fn io_error(err: IOError) -> Result<LoadResult<TAsset, LoadError>, UnreachableError> {
		Ok(LoadResult::Err(LoadError::IO(err)))
	}

	fn char_error(err: Utf8Error) -> Result<LoadResult<TAsset, LoadError>, UnreachableError> {
		Ok(LoadResult::Err(LoadError::ParseChars(err)))
	}

	fn parse_error(err: SerdeJsonError) -> Result<LoadResult<TAsset, LoadError>, UnreachableError> {
		Ok(LoadResult::Err(LoadError::ParseObject(err)))
	}

	fn ok(asset: TAsset) -> Result<LoadResult<TAsset, LoadError>, UnreachableError> {
		Ok(LoadResult::Ok(asset))
	}
}

#[derive(Debug, TypePath)]
#[allow(dead_code)]
pub(crate) enum LoadError {
	IO(IOError),
	ParseChars(Utf8Error),
	ParseObject(SerdeJsonError),
}

impl Display for LoadError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			LoadError::IO(err) => write!(f, "Failed to read asset file: {}", err),
			LoadError::ParseChars(err) => {
				write!(f, "Invalid character encoding in asset file: {}", err)
			}
			LoadError::ParseObject(err) => write!(f, "Failed to parse asset data: {}", err),
		}
	}
}

impl Error for LoadError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			LoadError::IO(err) => Some(err),
			LoadError::ParseChars(err) => Some(err),
			LoadError::ParseObject(err) => Some(err),
		}
	}
}

#[derive(Debug, TypePath)]
pub(crate) enum ReadError {
	IO(IOError),
	ParseChars(Utf8Error),
}

#[derive(Debug)]
pub(crate) struct UnreachableError;

impl Display for UnreachableError {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		write!(f, "{:?}: If you see this, the universe broke", self)
	}
}

impl Error for UnreachableError {}
