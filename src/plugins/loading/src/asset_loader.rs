use crate::folder_asset_loader::{LoadError, ReadError};
use bevy::asset::{io::Reader, Asset, AssetLoader, AsyncReadExt, LoadContext};
use common::traits::register_custom_assets::{AssetFileExtensions, LoadFrom};
use serde::Deserialize;
use std::{marker::PhantomData, str::from_utf8};

pub(crate) struct CustomAssetLoader<TAsset, TDto> {
	phantom_data: PhantomData<(TAsset, TDto)>,
}

impl<TAsset, TDto> CustomAssetLoader<TAsset, TDto> {
	async fn read<'a>(
		reader: &'a mut Reader<'_>,
		buffer: &'a mut Vec<u8>,
	) -> Result<&'a str, ReadError> {
		reader.read_to_end(buffer).await.map_err(ReadError::IO)?;
		from_utf8(buffer).map_err(ReadError::ParseChars)
	}
}

impl<TAsset, TDto> Default for CustomAssetLoader<TAsset, TDto> {
	fn default() -> Self {
		Self {
			phantom_data: PhantomData,
		}
	}
}

impl<TAsset, TDto> AssetLoader for CustomAssetLoader<TAsset, TDto>
where
	TAsset: Asset + LoadFrom<TDto>,
	for<'a> TDto: Deserialize<'a> + AssetFileExtensions + Sync + Send + 'static,
{
	type Asset = TAsset;
	type Settings = ();
	type Error = LoadError;

	fn extensions(&self) -> &[&str] {
		TDto::asset_file_extensions()
	}

	async fn load<'a>(
		&'a self,
		reader: &'a mut Reader<'_>,
		_: &'a Self::Settings,
		context: &'a mut LoadContext<'_>,
	) -> Result<Self::Asset, Self::Error> {
		let buffer = &mut vec![];

		let dto = match Self::read(reader, buffer).await {
			Err(ReadError::IO(err)) => return Err(LoadError::IO(err)),
			Err(ReadError::ParseChars(err)) => return Err(LoadError::ParseChars(err)),
			Ok(str) => serde_json::from_str(str),
		};

		match dto {
			Ok(dto) => Ok(TAsset::load_from(dto, context)),
			Err(err) => Err(LoadError::ParseObject(err)),
		}
	}
}
