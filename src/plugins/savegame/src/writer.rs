use crate::traits::write_to_file::WriteToFile;
use std::{fs, io::Error};

#[derive(Debug, PartialEq)]
pub struct FileWriter {
	destination: &'static str,
}

impl FileWriter {
	pub(crate) fn to_destination(destination: &'static str) -> Self {
		Self { destination }
	}
}

impl WriteToFile for FileWriter {
	type TError = Error;

	fn write(&self, string: String) -> Result<(), Self::TError> {
		fs::write(self.destination, string)
	}
}
