use bevy::ecs::entity::Entity;
use common::errors::{Error, Level};
use serde_json::Error as SerdeJsonError;
use std::collections::HashMap;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub(crate) enum ContextIOError<TIOError> {
	FileError(TIOError),
	SerdeErrors(SerdeJsonErrors),
	LockPoisoned(LockPoisonedError),
}

impl<TError> From<ContextIOError<TError>> for Error
where
	TError: Into<Error>,
{
	fn from(value: ContextIOError<TError>) -> Self {
		match value {
			ContextIOError::FileError(error) => error.into(),
			ContextIOError::SerdeErrors(error) => Self::from(error),
			ContextIOError::LockPoisoned(error) => Self::from(error),
		}
	}
}

#[derive(Debug)]
pub(crate) enum SerializationOrLockError {
	SerializationErrors(SerializationErrors),
	LockPoisoned(LockPoisonedError),
}

impl From<SerializationOrLockError> for Error {
	fn from(value: SerializationOrLockError) -> Self {
		match value {
			SerializationOrLockError::SerializationErrors(error) => Self::from(error),
			SerializationOrLockError::LockPoisoned(error) => Self::from(error),
		}
	}
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub(crate) enum DeserializationOrLockError {
	DeserializationErrors(SerdeJsonErrors),
	LockPoisoned(LockPoisonedError),
}

impl From<DeserializationOrLockError> for Error {
	fn from(value: DeserializationOrLockError) -> Self {
		match value {
			DeserializationOrLockError::DeserializationErrors(error) => Self::from(error),
			DeserializationOrLockError::LockPoisoned(error) => Self::from(error),
		}
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct LockPoisonedError;

impl From<LockPoisonedError> for Error {
	fn from(_: LockPoisonedError) -> Self {
		Self::Single {
			msg: "lock poisoned".to_owned(),
			lvl: Level::Error,
		}
	}
}

#[derive(Debug)]
pub struct SerializationErrors(pub(crate) HashMap<Entity, EntitySerializationErrors>);

impl From<SerializationErrors> for Error {
	fn from(SerializationErrors(map): SerializationErrors) -> Self {
		let msg = map
			.iter()
			.flat_map(|(entity, EntitySerializationErrors(errors))| {
				errors.iter().map(move |error| format!("{entity}: {error}"))
			})
			.collect::<String>();

		Self::Single {
			msg,
			lvl: Level::Error,
		}
	}
}

#[derive(Debug)]
pub(crate) struct EntitySerializationErrors(pub(crate) Vec<SerdeJsonError>);

#[derive(Debug)]
pub(crate) struct SerdeJsonErrors(pub(crate) Vec<SerdeJsonError>);

#[cfg(test)]
impl PartialEq for SerdeJsonErrors {
	fn eq(&self, other: &Self) -> bool {
		if self.0.len() != other.0.len() {
			return false;
		}

		for error in &self.0 {
			let matches = |other: &SerdeJsonError| {
				error.line() == other.line()
					&& error.column() == other.column()
					&& error.classify() == other.classify()
			};
			if !other.0.iter().any(matches) {
				return false;
			}
		}

		true
	}
}

impl From<SerdeJsonErrors> for Error {
	fn from(SerdeJsonErrors(errors): SerdeJsonErrors) -> Self {
		let errors = errors
			.iter()
			.map(|error| format!("- {error}"))
			.collect::<Vec<_>>()
			.join("\n");
		Self::Single {
			msg: format!("Failed to serialize data:\n{errors}"),
			lvl: Level::Error,
		}
	}
}
