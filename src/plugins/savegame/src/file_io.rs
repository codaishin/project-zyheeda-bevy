use crate::traits::{read_file::ReadFile, write_file::WriteFile};
use std::{
	fs,
	io::Error,
	path::{Path, PathBuf},
};

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct FileIO<TIO = IO>
where
	TIO: IOWrite + IORead,
{
	file: PathBuf,
	io: TIO,
}

impl FileIO {
	pub(crate) fn with_file(file: PathBuf) -> Self {
		Self { file, io: IO }
	}
}

impl<TIO> WriteFile for FileIO<TIO>
where
	TIO: IOWrite + IORead,
{
	type TError = TIO::TError;

	fn write(&self, string: &str) -> Result<(), Self::TError> {
		let path = self.file.as_path();
		let path_tmp = path.with_extension("tmp");
		let path_old = path.with_extension("old");

		if let Some(parent) = path.parent() {
			self.io.create_dir_all(parent)?;
		}

		self.io.write(&path_tmp, string)?;

		if self.io.exists(path) {
			self.io.rename(path, &path_old)?;
		}

		self.io.rename(&path_tmp, path)?;

		Ok(())
	}
}

impl ReadFile for FileIO {
	type TError = Error;

	fn read(&self) -> Result<String, Self::TError> {
		fs::read_to_string(self.file.as_path())
	}
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct IO;

pub(crate) trait IOWrite {
	type TError;

	fn create_dir_all(&self, path: &Path) -> Result<(), Self::TError>;
	fn write(&self, path: &Path, content: &str) -> Result<(), Self::TError>;
	fn rename(&self, from: &Path, to: &Path) -> Result<(), Self::TError>;
}

impl IOWrite for IO {
	type TError = Error;

	fn create_dir_all(&self, path: &Path) -> Result<(), Error> {
		fs::create_dir_all(path)
	}

	fn write(&self, path: &Path, content: &str) -> Result<(), Error> {
		fs::write(path, content)
	}

	fn rename(&self, from: &Path, to: &Path) -> Result<(), Error> {
		fs::rename(from, to)
	}
}

pub(crate) trait IORead {
	fn exists(&self, path: &Path) -> bool;
}

impl IORead for IO {
	fn exists(&self, path: &Path) -> bool {
		path.exists()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{simple_init, traits::mock::Mock};
	use mockall::{Sequence, mock};

	mock! {
		_IO {}
		impl IOWrite for _IO {
			type TError = _Error;

			fn create_dir_all(&self, path: &Path) -> Result<(), _Error>;
			fn write(&self, path: &Path, content: &str) -> Result<(), _Error>;
			fn rename(&self, from: &Path, to: &Path) -> Result<(), _Error>;
		}
		impl IORead for _IO {
			fn exists(&self, path: &Path) -> bool;
		}
	}

	simple_init!(Mock_IO);

	struct _Error;

	#[test]
	fn create_parent_directory() {
		let file_io = FileIO {
			file: PathBuf::from("/my/path/to/file.json"),
			io: Mock_IO::new_mock(|mock| {
				mock.expect_create_dir_all().times(1).returning(|path| {
					assert_eq!(Some("/my/path/to"), path.to_str());
					Ok(())
				});
				mock.expect_write().returning(|_, _| Ok(()));
				mock.expect_exists().return_const(true);
				mock.expect_rename().returning(|_, _| Ok(()));
			}),
		};

		_ = file_io.write("");
	}

	#[test]
	fn return_create_dir_error() {
		let file_io = FileIO {
			file: PathBuf::from("/my/path/to/file.json"),
			io: Mock_IO::new_mock(|mock| {
				mock.expect_create_dir_all().returning(|_| Err(_Error));
				mock.expect_write().returning(|_, _| Ok(()));
				mock.expect_exists().return_const(true);
				mock.expect_rename().returning(|_, _| Ok(()));
			}),
		};

		assert!(file_io.write("").is_err());
	}

	#[test]
	fn write_content() {
		let file_io = FileIO {
			file: PathBuf::from("/my/path/to/file.json"),
			io: Mock_IO::new_mock(|mock| {
				mock.expect_create_dir_all().returning(|_| Ok(()));
				mock.expect_write().times(1).returning(|path, content| {
					assert_eq!(
						(Some("/my/path/to/file.tmp"), "content"),
						(path.to_str(), content)
					);
					Ok(())
				});
				mock.expect_exists().return_const(true);
				mock.expect_rename().returning(|_, _| Ok(()));
			}),
		};

		_ = file_io.write("content");
	}

	#[test]
	fn return_write_error() {
		let file_io = FileIO {
			file: PathBuf::from("/my/path/to/file.json"),
			io: Mock_IO::new_mock(|mock| {
				mock.expect_create_dir_all().returning(|_| Ok(()));
				mock.expect_write().times(1).returning(|_, _| Err(_Error));
				mock.expect_exists().return_const(true);
				mock.expect_rename().returning(|_, _| Ok(()));
			}),
		};

		assert!(file_io.write("").is_err());
	}

	#[test]
	fn rename_files() {
		let file_io = FileIO {
			file: PathBuf::from("/my/path/to/file.json"),
			io: Mock_IO::new_mock(|mock| {
				mock.expect_create_dir_all().returning(|_| Ok(()));
				mock.expect_write().times(1).returning(|_, _| Ok(()));
				mock.expect_exists().return_const(true);
				mock.expect_rename()
					.times(1)
					.withf(|from, to| {
						(Some("/my/path/to/file.json"), Some("/my/path/to/file.old"))
							== (from.to_str(), to.to_str())
					})
					.returning(|_, _| Ok(()));
				mock.expect_rename()
					.times(1)
					.withf(|from, to| {
						(Some("/my/path/to/file.tmp"), Some("/my/path/to/file.json"))
							== (from.to_str(), to.to_str())
					})
					.returning(|_, _| Ok(()));
			}),
		};

		_ = file_io.write("");
	}

	#[test]
	fn do_not_rename_file_if_it_does_not_exist() {
		let file_io = FileIO {
			file: PathBuf::from("/my/path/to/file.json"),
			io: Mock_IO::new_mock(|mock| {
				mock.expect_create_dir_all().returning(|_| Ok(()));
				mock.expect_write().times(1).returning(|_, _| Ok(()));
				mock.expect_exists()
					.times(1)
					.withf(|path| Some("/my/path/to/file.json") == path.to_str())
					.return_const(false);
				mock.expect_rename()
					.never()
					.withf(|from, to| {
						(Some("/my/path/to/file.json"), Some("/my/path/to/file.old"))
							== (from.to_str(), to.to_str())
					})
					.returning(|_, _| Ok(()));
				mock.expect_rename()
					.times(1)
					.withf(|from, to| {
						(Some("/my/path/to/file.tmp"), Some("/my/path/to/file.json"))
							== (from.to_str(), to.to_str())
					})
					.returning(|_, _| Ok(()));
			}),
		};

		_ = file_io.write("");
	}

	#[test]
	fn return_first_rename_error() {
		let file_io = FileIO {
			file: PathBuf::from("/my/path/to/file.json"),
			io: Mock_IO::new_mock(|mock| {
				mock.expect_create_dir_all().returning(|_| Ok(()));
				mock.expect_write().times(1).returning(|_, _| Ok(()));
				mock.expect_exists().return_const(true);
				mock.expect_rename()
					.withf(|from, to| {
						(Some("/my/path/to/file.json"), Some("/my/path/to/file.old"))
							== (from.to_str(), to.to_str())
					})
					.returning(|_, _| Err(_Error));
				mock.expect_rename()
					.withf(|from, to| {
						(Some("/my/path/to/file.tmp"), Some("/my/path/to/file.json"))
							== (from.to_str(), to.to_str())
					})
					.returning(|_, _| Ok(()));
			}),
		};

		assert!(file_io.write("").is_err());
	}

	#[test]
	fn return_second_rename_error() {
		let file_io = FileIO {
			file: PathBuf::from("/my/path/to/file.json"),
			io: Mock_IO::new_mock(|mock| {
				mock.expect_create_dir_all().returning(|_| Ok(()));
				mock.expect_write().times(1).returning(|_, _| Ok(()));
				mock.expect_exists().return_const(true);
				mock.expect_rename()
					.withf(|from, to| {
						(Some("/my/path/to/file.json"), Some("/my/path/to/file.old"))
							== (from.to_str(), to.to_str())
					})
					.returning(|_, _| Ok(()));
				mock.expect_rename()
					.withf(|from, to| {
						(Some("/my/path/to/file.tmp"), Some("/my/path/to/file.json"))
							== (from.to_str(), to.to_str())
					})
					.returning(|_, _| Err(_Error));
			}),
		};

		assert!(file_io.write("").is_err());
	}

	#[test]
	fn call_functions_in_sequence() {
		let file_io = FileIO {
			file: PathBuf::from("/my/path/to/file.json"),
			io: Mock_IO::new_mock(|mock| {
				let sequence = &mut Sequence::new();
				mock.expect_create_dir_all()
					.times(1)
					.in_sequence(sequence)
					.returning(|_| Ok(()));
				mock.expect_write()
					.times(1)
					.in_sequence(sequence)
					.returning(|_, _| Ok(()));
				mock.expect_exists()
					.times(1)
					.in_sequence(sequence)
					.return_const(true);
				mock.expect_rename()
					.times(1)
					.in_sequence(sequence)
					.withf(|from, to| {
						(Some("/my/path/to/file.json"), Some("/my/path/to/file.old"))
							== (from.to_str(), to.to_str())
					})
					.returning(|_, _| Ok(()));
				mock.expect_rename()
					.times(1)
					.in_sequence(sequence)
					.withf(|from, to| {
						(Some("/my/path/to/file.tmp"), Some("/my/path/to/file.json"))
							== (from.to_str(), to.to_str())
					})
					.returning(|_, _| Ok(()));
			}),
		};

		_ = file_io.write("");
	}
}
