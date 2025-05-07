use crate::folder_asset_loader::{LoadError, ReadError};
use bevy::{
	asset::{Asset, AssetLoader, LoadContext, io::Reader},
	reflect::TypePath,
};
use common::traits::{
	handles_custom_assets::{AssetFileExtensions, TryLoadFrom},
	thread_safe::ThreadSafe,
};
use serde::Deserialize;
use std::{error::Error, marker::PhantomData, str::from_utf8};

pub(crate) struct CustomAssetLoader<TAsset, TDto> {
	phantom_data: PhantomData<(TAsset, TDto)>,
}

impl<TAsset, TDto> CustomAssetLoader<TAsset, TDto> {
	async fn read<'a>(
		reader: &mut dyn Reader,
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
	TAsset: Asset + TryLoadFrom<TDto>,
	TAsset::TInstantiationError: Error + TypePath + ThreadSafe,
	for<'a> TDto: Deserialize<'a> + AssetFileExtensions + Sync + Send + 'static,
{
	type Asset = TAsset;
	type Settings = ();
	type Error = LoadError<TAsset::TInstantiationError>;

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
			Err(ReadError::IO(err)) => return Err(LoadError::IO(err)),
			Err(ReadError::ParseChars(err)) => return Err(LoadError::ParseChars(err)),
			Ok(str) => serde_json::from_str(str),
		};

		let loaded = match dto {
			Ok(dto) => TAsset::try_load_from(dto, context),
			Err(err) => return Err(LoadError::ParseObject(err)),
		};

		match loaded {
			Ok(loaded) => Ok(loaded),
			Err(err) => Err(LoadError::Instantiation(err)),
		}
	}
}
