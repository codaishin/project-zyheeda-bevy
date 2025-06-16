use crate::traits::write_to_file::WriteToFile;
use std::{fs, io::Error, path::PathBuf};

#[derive(Debug, PartialEq)]
pub struct FileWriter {
	destination: PathBuf,
}

impl FileWriter {
	pub(crate) fn to_destination(destination: PathBuf) -> Self {
		Self { destination }
	}
}

impl WriteToFile for FileWriter {
	type TError = Error;

	fn write(&self, string: &str) -> Result<(), Self::TError> {
		let path = self.destination.as_path();

		if let Some(parent) = path.parent() {
			fs::create_dir_all(parent)?;
		}

		fs::write(path, string)
	}
}
