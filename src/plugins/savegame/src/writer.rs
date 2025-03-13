use crate::traits::write_to_file::WriteToFile;
use std::{fs, io::Error};

#[derive(Debug, PartialEq)]
pub struct FileWriter {
	pub(crate) destination: &'static str,
}

impl WriteToFile for FileWriter {
	type TError = Error;

	fn write(&self, string: String) -> Result<(), Self::TError> {
		fs::write(self.destination, string)
	}
}
