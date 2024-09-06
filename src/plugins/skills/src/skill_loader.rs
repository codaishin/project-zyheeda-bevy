use crate::skills::skill_data::SkillData;
use bevy::{
	asset::{io::Reader, Asset, AssetLoader, AsyncReadExt, LoadContext},
	reflect::TypePath,
};
use common::traits::load_from::LoadFrom;
use serde_json::error::Error as SerdeJsonError;
use std::{
	error::Error,
	fmt::{Display, Formatter, Result as FmtResult},
	io::Error as IOError,
	marker::PhantomData,
	str::{from_utf8, Utf8Error},
};

/// Skill loader that always returns `Ok`. Errors are stored within the `Ok`
/// side of the result, so we can handle them on an individual level and prevent
/// bevy from stopping the load process when loading the whole skill folder.
pub struct SkillLoader<TSkill> {
	phantom_data: PhantomData<TSkill>,
}

impl<TSkill> Default for SkillLoader<TSkill> {
	fn default() -> Self {
		Self {
			phantom_data: PhantomData,
		}
	}
}

#[derive(Debug, TypePath)]
#[allow(dead_code)]
pub enum LoadError {
	IO(IOError),
	ParseChars(Utf8Error),
	ParseObject(SerdeJsonError),
}

#[derive(Debug)]
pub struct UnreachableError;

impl Display for UnreachableError {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		write!(f, "{:?}: If you see this, the universe broke", self)
	}
}

impl Error for UnreachableError {}

#[derive(Asset, TypePath, Debug, PartialEq)]
pub enum LoadResult<TAsset: Asset, TError: Sync + Send + TypePath + 'static = LoadError> {
	Ok(TAsset),
	Err(TError),
}

impl<TAsset: Asset, TError: Sync + Send + TypePath + 'static> LoadResult<TAsset, TError> {
	fn error<TOuterError>(error: TError) -> Result<LoadResult<TAsset, TError>, TOuterError> {
		Ok(LoadResult::Err(error))
	}

	fn ok<TOuterError>(asset: TAsset) -> Result<LoadResult<TAsset, TError>, TOuterError> {
		Ok(LoadResult::Ok(asset))
	}
}

impl<TAsset: Asset + LoadFrom<SkillData>> AssetLoader for SkillLoader<LoadResult<TAsset>> {
	type Asset = LoadResult<TAsset>;
	type Settings = ();
	type Error = UnreachableError;

	fn extensions(&self) -> &[&str] {
		&["skill"]
	}

	async fn load<'a>(
		&'a self,
		reader: &'a mut Reader<'_>,
		_: &'a Self::Settings,
		load_context: &'a mut LoadContext<'_>,
	) -> Result<Self::Asset, Self::Error> {
		let mut bytes = Vec::new();
		let result = reader
			.read_to_end(&mut bytes)
			.await
			.map(|_| from_utf8(&bytes));

		let data = match result {
			Err(io_error) => return LoadResult::error(LoadError::IO(io_error)),
			Ok(Err(utf8_error)) => return LoadResult::error(LoadError::ParseChars(utf8_error)),
			Ok(Ok(str)) => serde_json::from_str(str),
		};

		match data {
			Ok(data) => LoadResult::ok(TAsset::load_from(data, load_context)),
			Err(error) => LoadResult::error(LoadError::ParseObject(error)),
		}
	}
}
