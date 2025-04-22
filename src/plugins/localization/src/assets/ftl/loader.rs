use super::Ftl;
use bevy::{
	asset::{AssetLoader, LoadContext, io::Reader},
	prelude::*,
};
use std::{
	error::Error,
	fmt::{Display, Formatter, Result as FmtResult},
	io::Error as IOError,
	str::{Utf8Error, from_utf8},
};

pub(crate) struct FtlLoader;

impl FtlLoader {
	async fn read<'a>(
		reader: &mut dyn Reader,
		buffer: &'a mut Vec<u8>,
	) -> Result<&'a str, ReadError> {
		reader.read_to_end(buffer).await.map_err(ReadError::IO)?;
		from_utf8(buffer).map_err(ReadError::ParseChars)
	}
}

impl AssetLoader for FtlLoader {
	type Asset = Ftl;
	type Settings = ();
	type Error = ReadError;

	fn extensions(&self) -> &[&str] {
		&[".ftl"]
	}

	async fn load(
		&self,
		reader: &mut dyn Reader,
		_: &Self::Settings,
		_: &mut LoadContext<'_>,
	) -> Result<Self::Asset, Self::Error> {
		let buffer = &mut vec![];
		let content = Self::read(reader, buffer).await?;

		Ok(Ftl(content.to_owned()))
	}
}

#[derive(Debug, TypePath)]
pub enum ReadError {
	IO(IOError),
	ParseChars(Utf8Error),
}

impl Display for ReadError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			ReadError::IO(error) => write!(f, "ReadError::IO: {error}"),
			ReadError::ParseChars(error) => write!(f, "ReadError::ParseChars: {error}"),
		}
	}
}

impl Error for ReadError {}
