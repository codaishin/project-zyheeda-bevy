use bevy::prelude::*;
use common::{
	errors::{ErrorData, Level},
	tools::path::Path,
};
use serde::Serialize;
use serde_json::{Error as JsonError, to_string_pretty};
use std::{
	fmt::Display,
	fs::File,
	io::{Error as IoError, Write},
	path::PathBuf,
};

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct AssetWriter {
	asset_path: PathBuf,
}

impl AssetWriter {
	fn open_for_override(&self, path: Path) -> Result<File, IoError> {
		let path = self.asset_path.join(path.path());
		File::create(path)
	}
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
		let mut file = match self.open_for_override(path) {
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

impl Display for WriteError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			WriteError::Serde(error) => write!(f, "failed to serialize asset: {error}"),
			WriteError::Io(error) => write!(f, "failed to save asset: {error}"),
		}
	}
}

impl ErrorData for WriteError {
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl Display {
		"Write operation failed"
	}

	fn into_details(self) -> impl Display {
		self
	}
}
