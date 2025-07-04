use crate::traits::{file_exists::FileExists, read_file::ReadFile, write_file::WriteFile};
use common::errors::{Error, Level};
use std::{
	ffi::OsStr,
	fs,
	io::Error as IOError,
	path::{Path, PathBuf},
};

mod prefixes {
	pub const TMP: &str = "tmp";
	pub const OLD: &str = "old";
}

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

impl<TIO> FileIO<TIO>
where
	TIO: IOWrite + IORead,
{
	fn file_does_not_exist(&self) -> FileError<TIO::TReadError> {
		let core_path = self.file.with_extension("");
		let core_path = core_path.to_str().unwrap_or("invalid path");
		let main_extension = self.file.extension().and_then(OsStr::to_str);
		let old = prefixes::OLD;
		let tmp = prefixes::TMP;

		let extensions = match main_extension {
			Some(ext) => format!(".{ext}|.{old}.{ext}|.{tmp}.{ext}"),
			None => format!("|.{old}|.{tmp}"),
		};

		FileError::DoesNotExist(format!("{core_path}({extensions})"))
	}

	fn file_variations(&self) -> impl IntoIterator<Item = PathBuf> {
		[
			self.file.clone(),
			self.file.with_extension_prefix(prefixes::TMP),
			self.file.with_extension_prefix(prefixes::OLD),
		]
	}
}

impl<TIO> WriteFile for FileIO<TIO>
where
	TIO: IOWrite + IORead,
{
	type TWriteError = TIO::TWriteError;

	fn write(&self, string: &str) -> Result<(), Self::TWriteError> {
		let path = &self.file;
		let path_tmp = path.with_extension_prefix(prefixes::TMP);
		let path_old = path.with_extension_prefix(prefixes::OLD);

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

impl<TIO> ReadFile for FileIO<TIO>
where
	TIO: IOWrite + IORead,
{
	type TReadError = FileError<TIO::TReadError>;

	fn read(&self) -> Result<String, Self::TReadError> {
		for path in self.file_variations() {
			if !self.io.exists(&path) {
				continue;
			}
			return self.io.read_to_string(&path).map_err(FileError::IO);
		}

		Err(self.file_does_not_exist())
	}
}

impl<TIO> FileExists for FileIO<TIO>
where
	TIO: IOWrite + IORead,
{
	fn file_exists(&self) -> bool {
		self.file_variations()
			.into_iter()
			.any(|file| self.io.exists(&file))
	}
}

#[derive(Debug, PartialEq)]
pub(crate) enum FileError<TError = IOError> {
	IO(TError),
	DoesNotExist(String),
}

impl From<FileError> for Error {
	fn from(error: FileError) -> Self {
		match error {
			FileError::IO(error) => Self::from(error),
			FileError::DoesNotExist(save_game) => Error {
				msg: format!("`{save_game}`: not found",),
				lvl: Level::Warning,
			},
		}
	}
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct IO;

pub(crate) trait IOWrite {
	type TWriteError;

	fn create_dir_all(&self, path: &Path) -> Result<(), Self::TWriteError>;
	fn write(&self, path: &Path, content: &str) -> Result<(), Self::TWriteError>;
	fn rename(&self, from: &Path, to: &Path) -> Result<(), Self::TWriteError>;
}

impl IOWrite for IO {
	type TWriteError = IOError;

	fn create_dir_all(&self, path: &Path) -> Result<(), IOError> {
		fs::create_dir_all(path)
	}

	fn write(&self, path: &Path, content: &str) -> Result<(), IOError> {
		fs::write(path, content)
	}

	fn rename(&self, from: &Path, to: &Path) -> Result<(), IOError> {
		fs::rename(from, to)
	}
}

pub(crate) trait IORead {
	type TReadError;

	fn exists(&self, path: &Path) -> bool;
	fn read_to_string(&self, path: &Path) -> Result<String, Self::TReadError>;
}

impl IORead for IO {
	type TReadError = IOError;

	fn exists(&self, path: &Path) -> bool {
		path.exists()
	}

	fn read_to_string(&self, path: &Path) -> Result<String, Self::TReadError> {
		fs::read_to_string(path)
	}
}

trait WithExtensionPrefix {
	/// Prefixes extension
	///
	/// With the prefix literal `"prefix"` `/my/path.json` will become `/my/path.prefix.json`.
	///
	/// The prefix will be used as the extension if
	/// - No extension exists
	/// - Converting the extension to a `&str` fails
	fn with_extension_prefix(&self, prefix: &str) -> Self;
}

impl WithExtensionPrefix for PathBuf {
	fn with_extension_prefix(&self, prefix: &str) -> Self {
		let new_extension = match self.extension().and_then(OsStr::to_str) {
			Some(extension) => format!("{prefix}.{extension}"),
			None => prefix.to_owned(),
		};

		self.with_extension(new_extension)
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
			type TWriteError = _Error;

			fn create_dir_all(&self, path: &Path) -> Result<(), _Error>;
			fn write(&self, path: &Path, content: &str) -> Result<(), _Error>;
			fn rename(&self, from: &Path, to: &Path) -> Result<(), _Error>;
		}
		impl IORead for _IO {
			type TReadError = _Error;

			fn exists(&self, path: &Path) -> bool;
			fn read_to_string(&self ,path: &Path) -> Result<String, _Error>;
		}
	}

	simple_init!(Mock_IO);

	#[derive(Debug, PartialEq)]
	struct _Error;

	mod write {
		use super::*;
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
							(Some("/my/path/to/file.tmp.json"), "content"),
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
							(
								Some("/my/path/to/file.json"),
								Some("/my/path/to/file.old.json"),
							) == (from.to_str(), to.to_str())
						})
						.returning(|_, _| Ok(()));
					mock.expect_rename()
						.times(1)
						.withf(|from, to| {
							(
								Some("/my/path/to/file.tmp.json"),
								Some("/my/path/to/file.json"),
							) == (from.to_str(), to.to_str())
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
							(
								Some("/my/path/to/file.json"),
								Some("/my/path/to/file.old.json"),
							) == (from.to_str(), to.to_str())
						})
						.returning(|_, _| Ok(()));
					mock.expect_rename()
						.times(1)
						.withf(|from, to| {
							(
								Some("/my/path/to/file.tmp.json"),
								Some("/my/path/to/file.json"),
							) == (from.to_str(), to.to_str())
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
							(
								Some("/my/path/to/file.json"),
								Some("/my/path/to/file.old.json"),
							) == (from.to_str(), to.to_str())
						})
						.returning(|_, _| Err(_Error));
					mock.expect_rename()
						.withf(|from, to| {
							(
								Some("/my/path/to/file.tmp.json"),
								Some("/my/path/to/file.json"),
							) == (from.to_str(), to.to_str())
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
							(
								Some("/my/path/to/file.json"),
								Some("/my/path/to/file.old.json"),
							) == (from.to_str(), to.to_str())
						})
						.returning(|_, _| Ok(()));
					mock.expect_rename()
						.withf(|from, to| {
							(
								Some("/my/path/to/file.tmp.json"),
								Some("/my/path/to/file.json"),
							) == (from.to_str(), to.to_str())
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
							(
								Some("/my/path/to/file.json"),
								Some("/my/path/to/file.old.json"),
							) == (from.to_str(), to.to_str())
						})
						.returning(|_, _| Ok(()));
					mock.expect_rename()
						.times(1)
						.in_sequence(sequence)
						.withf(|from, to| {
							(
								Some("/my/path/to/file.tmp.json"),
								Some("/my/path/to/file.json"),
							) == (from.to_str(), to.to_str())
						})
						.returning(|_, _| Ok(()));
				}),
			};

			_ = file_io.write("");
		}
	}

	mod read {
		use super::*;

		#[test]
		fn read_file() {
			let file_io = FileIO {
				file: PathBuf::from("/my/path/to/file.json"),
				io: Mock_IO::new_mock(|mock| {
					mock.expect_exists().return_const(true);
					mock.expect_read_to_string()
						.times(1)
						.withf(|path| Some("/my/path/to/file.json") == path.to_str())
						.returning(|_| Ok(String::from("content")));
				}),
			};

			assert_eq!(Ok(String::from("content")), file_io.read());
		}

		#[test]
		fn read_file_error() {
			let file_io = FileIO {
				file: PathBuf::from("/my/path/to/file.json"),
				io: Mock_IO::new_mock(|mock| {
					mock.expect_exists().return_const(true);
					mock.expect_read_to_string()
						.times(1)
						.withf(|path| Some("/my/path/to/file.json") == path.to_str())
						.returning(|_| Err(_Error));
				}),
			};

			assert_eq!(Err(FileError::IO(_Error)), file_io.read());
		}

		#[test]
		fn read_tmp_file_if_path_does_not_exist() {
			let file_io = FileIO {
				file: PathBuf::from("/my/path/to/file.json"),
				io: Mock_IO::new_mock(|mock| {
					mock.expect_exists()
						.withf(|path| Some("/my/path/to/file.json") == path.to_str())
						.return_const(false);
					mock.expect_exists()
						.withf(|path| Some("/my/path/to/file.tmp.json") == path.to_str())
						.return_const(true);
					mock.expect_read_to_string()
						.times(1)
						.withf(|path| Some("/my/path/to/file.tmp.json") == path.to_str())
						.returning(|_| Ok(String::from("content tmp")));
				}),
			};

			assert_eq!(Ok(String::from("content tmp")), file_io.read());
		}

		#[test]
		fn read_tmp_file_error() {
			let file_io = FileIO {
				file: PathBuf::from("/my/path/to/file.json"),
				io: Mock_IO::new_mock(|mock| {
					mock.expect_exists()
						.withf(|path| Some("/my/path/to/file.json") == path.to_str())
						.return_const(false);
					mock.expect_exists()
						.withf(|path| Some("/my/path/to/file.tmp.json") == path.to_str())
						.return_const(true);
					mock.expect_read_to_string()
						.times(1)
						.withf(|path| Some("/my/path/to/file.tmp.json") == path.to_str())
						.returning(|_| Err(_Error));
				}),
			};

			assert_eq!(Err(FileError::IO(_Error)), file_io.read());
		}

		#[test]
		fn read_old_file_if_tmp_path_does_not_exist() {
			let file_io = FileIO {
				file: PathBuf::from("/my/path/to/file.json"),
				io: Mock_IO::new_mock(|mock| {
					mock.expect_exists()
						.withf(|path| Some("/my/path/to/file.json") == path.to_str())
						.return_const(false);
					mock.expect_exists()
						.withf(|path| Some("/my/path/to/file.tmp.json") == path.to_str())
						.return_const(false);
					mock.expect_exists()
						.withf(|path| Some("/my/path/to/file.old.json") == path.to_str())
						.return_const(true);
					mock.expect_read_to_string()
						.times(1)
						.withf(|path| Some("/my/path/to/file.old.json") == path.to_str())
						.returning(|_| Ok(String::from("content old")));
				}),
			};

			assert_eq!(Ok(String::from("content old")), file_io.read());
		}

		#[test]
		fn read_old_file_error() {
			let file_io = FileIO {
				file: PathBuf::from("/my/path/to/file.json"),
				io: Mock_IO::new_mock(|mock| {
					mock.expect_exists()
						.withf(|path| Some("/my/path/to/file.json") == path.to_str())
						.return_const(false);
					mock.expect_exists()
						.withf(|path| Some("/my/path/to/file.tmp.json") == path.to_str())
						.return_const(false);
					mock.expect_exists()
						.withf(|path| Some("/my/path/to/file.old.json") == path.to_str())
						.return_const(true);
					mock.expect_read_to_string()
						.times(1)
						.withf(|path| Some("/my/path/to/file.old.json") == path.to_str())
						.returning(|_| Err(_Error));
				}),
			};

			assert_eq!(Err(FileError::IO(_Error)), file_io.read());
		}

		#[test]
		fn save_game_not_found_error() {
			let file_io = FileIO {
				file: PathBuf::from("/my/path/to/file.json"),
				io: Mock_IO::new_mock(|mock| {
					mock.expect_exists().return_const(false);
				}),
			};

			assert_eq!(
				Err(FileError::DoesNotExist(
					"/my/path/to/file(.json|.old.json|.tmp.json)".to_owned()
				)),
				file_io.read()
			);
		}
	}

	mod file_exists {
		use super::*;

		#[test]
		fn save_file_exists() {
			let io = FileIO {
				file: PathBuf::from("/my/file.json"),
				io: Mock_IO::new_mock(|mock| {
					mock.expect_exists()
						.withf(|path| Some("/my/file.json") == path.to_str())
						.return_const(true);
					mock.expect_exists()
						.withf(|path| Some("/my/file.tmp.json") == path.to_str())
						.return_const(false);
					mock.expect_exists()
						.withf(|path| Some("/my/file.old.json") == path.to_str())
						.return_const(false);
				}),
			};

			assert!(io.file_exists());
		}

		#[test]
		fn tmp_file_exists() {
			let io = FileIO {
				file: PathBuf::from("/my/file.json"),
				io: Mock_IO::new_mock(|mock| {
					mock.expect_exists()
						.withf(|path| Some("/my/file.json") == path.to_str())
						.return_const(false);
					mock.expect_exists()
						.withf(|path| Some("/my/file.tmp.json") == path.to_str())
						.return_const(true);
					mock.expect_exists()
						.withf(|path| Some("/my/file.old.json") == path.to_str())
						.return_const(false);
				}),
			};

			assert!(io.file_exists());
		}

		#[test]
		fn old_file_exists() {
			let io = FileIO {
				file: PathBuf::from("/my/file.json"),
				io: Mock_IO::new_mock(|mock| {
					mock.expect_exists()
						.withf(|path| Some("/my/file.json") == path.to_str())
						.return_const(false);
					mock.expect_exists()
						.withf(|path| Some("/my/file.tmp.json") == path.to_str())
						.return_const(false);
					mock.expect_exists()
						.withf(|path| Some("/my/file.old.json") == path.to_str())
						.return_const(true);
				}),
			};

			assert!(io.file_exists());
		}

		#[test]
		fn no_file_does_exist() {
			let io = FileIO {
				file: PathBuf::from("/my/file.json"),
				io: Mock_IO::new_mock(|mock| {
					mock.expect_exists()
						.withf(|path| Some("/my/file.json") == path.to_str())
						.return_const(false);
					mock.expect_exists()
						.withf(|path| Some("/my/file.tmp.json") == path.to_str())
						.return_const(false);
					mock.expect_exists()
						.withf(|path| Some("/my/file.old.json") == path.to_str())
						.return_const(false);
				}),
			};

			assert!(!io.file_exists());
		}
	}
}
