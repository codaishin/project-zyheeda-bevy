use bevy::prelude::*;
use common::{
	errors::{Error, Level},
	traits::load_asset::Path,
};
use serde::Serialize;
use serde_json::{Error as JsonError, to_string_pretty};
use std::{
	fs::File,
	io::{Error as IoError, Write},
	path::PathBuf,
};

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct AssetWriter {
	asset_path: PathBuf,
}

impl Default for AssetWriter {
	fn default() -> Self {
		Self {
			asset_path: PathBuf::from("assets"),
		}
	}
}

pub(crate) trait WriteAsset {
	type TError;

	fn write<TAsset>(&self, asset: TAsset, path: Path) -> Result<(), Self::TError>
	where
		TAsset: Serialize + 'static;
}

impl WriteAsset for AssetWriter {
	type TError = WriteError;

	fn write<TAsset>(&self, asset: TAsset, path: Path) -> Result<(), Self::TError>
	where
		TAsset: Serialize,
	{
		let string = match to_string_pretty(&asset) {
			Ok(string) => string,
			Err(err) => return Err(WriteError::Serde(err)),
		};
		let path = self.asset_path.join(path.as_string());
		let mut file = match File::open(path) {
			Ok(file) => file,
			Err(err) => return Err(WriteError::Io(err)),
		};

		match file.write_all(string.as_bytes()) {
			Ok(()) => Ok(()),
			Err(err) => Err(WriteError::Io(err)),
		}
	}
}

pub(crate) enum WriteError {
	Serde(JsonError),
	Io(IoError),
}

impl From<WriteError> for Error {
	fn from(value: WriteError) -> Self {
		match value {
			WriteError::Serde(error) => Error {
				msg: format!("failed to serialize asset: {error}"),
				lvl: Level::Error,
			},
			WriteError::Io(error) => Error {
				msg: format!("failed to save asset: {error}"),
				lvl: Level::Error,
			},
		}
	}
}
