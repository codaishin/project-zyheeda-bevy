use common::errors::{Error, Level};
use std::io::Error as IoError;

pub(crate) enum IoOrLockError<TIoError = IoError> {
	IoError(TIoError),
	LockPoisoned(LockPoisonedError),
}

impl From<IoOrLockError> for Error {
	fn from(value: IoOrLockError) -> Self {
		match value {
			IoOrLockError::IoError(error) => Self::from(error),
			IoOrLockError::LockPoisoned(error) => Self::from(error),
		}
	}
}

pub struct LockPoisonedError;

impl From<LockPoisonedError> for Error {
	fn from(_: LockPoisonedError) -> Self {
		Self {
			msg: "lock poisoned".to_owned(),
			lvl: Level::Error,
		}
	}
}
