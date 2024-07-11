use crate::skills::skill_data::SkillData;
use bevy::asset::{io::Reader, Asset, AssetLoader, AsyncReadExt, LoadContext};
use serde_json::error::Error as SerdeJsonError;
use std::{
	error::Error,
	fmt::{Display, Formatter, Result as FmtResult},
	io::Error as IOError,
	marker::PhantomData,
	str::{from_utf8, Utf8Error},
};

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

#[derive(Debug)]
pub enum SkillLoadError {
	IO(IOError),
	ParseChars(Utf8Error),
	ParseSkill(SerdeJsonError),
}

impl Display for SkillLoadError {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		match self {
			SkillLoadError::IO(error) => write!(f, "IO: {error}"),
			SkillLoadError::ParseChars(error) => write!(f, "ParseChars: {error}"),
			SkillLoadError::ParseSkill(error) => write!(f, "ParseSkill: {error}"),
		}
	}
}

impl Error for SkillLoadError {}

impl<TAsset: Asset + From<SkillData>> AssetLoader for SkillLoader<TAsset> {
	type Asset = TAsset;
	type Settings = ();
	type Error = SkillLoadError;

	fn extensions(&self) -> &[&str] {
		&["skill"]
	}

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
			Err(io_error) => Err(SkillLoadError::IO(io_error)),
			Ok(Err(utf8_error)) => Err(SkillLoadError::ParseChars(utf8_error)),
			Ok(Ok(str)) => serde_json::from_str(str)
				.map(TAsset::from)
				.map_err(SkillLoadError::ParseSkill),
		}
	}
}
