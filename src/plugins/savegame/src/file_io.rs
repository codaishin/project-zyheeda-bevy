use crate::traits::{read_file::ReadFile, write_file::WriteFile};
use std::{fs, io::Error, path::PathBuf};

#[derive(Debug, PartialEq)]
pub struct FileIO {
	file: PathBuf,
}

impl FileIO {
	pub(crate) fn with_file(file: PathBuf) -> Self {
		Self { file }
	}
}

impl WriteFile for FileIO {
	type TError = Error;

	fn write(&self, string: &str) -> Result<(), Self::TError> {
		let path = self.file.as_path();

		if let Some(parent) = path.parent() {
			fs::create_dir_all(parent)?;
		}

		fs::write(path, string)
	}
}

impl ReadFile for FileIO {
	type TError = Error;

	fn read(&self) -> Result<String, Self::TError> {
		fs::read_to_string(self.file.as_path())
	}
}
