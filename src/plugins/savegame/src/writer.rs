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

	fn write(&self, string: String) -> Result<(), Self::TError> {
		fs::write(self.destination.as_path(), string)
	}
}
