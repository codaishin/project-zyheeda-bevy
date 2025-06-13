use bevy::ecs::entity::Entity;
use common::errors::{Error, Level};
use serde_json::Error as SerdeJsonError;
use std::{collections::HashMap, io::Error as IoError};

#[derive(Debug)]
pub(crate) enum ContextFlushError<TIoError = IoError> {
	WriteError(TIoError),
	SerdeErrors(SerdeJsonErrors),
	LockPoisoned(LockPoisonedError),
}

impl From<ContextFlushError> for Error {
	fn from(value: ContextFlushError) -> Self {
		match value {
			ContextFlushError::WriteError(error) => Self::from(error),
			ContextFlushError::SerdeErrors(error) => Self::from(error),
			ContextFlushError::LockPoisoned(error) => Self::from(error),
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

#[derive(Debug, PartialEq, Clone)]
pub struct LockPoisonedError;

impl From<LockPoisonedError> for Error {
	fn from(_: LockPoisonedError) -> Self {
		Self {
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

		Self {
			msg,
			lvl: Level::Error,
		}
	}
}

#[derive(Debug)]
pub(crate) struct EntitySerializationErrors(pub(crate) Vec<SerdeJsonError>);

#[derive(Debug)]
pub(crate) struct SerdeJsonErrors(pub(crate) Vec<SerdeJsonError>);

impl From<SerdeJsonErrors> for Error {
	fn from(SerdeJsonErrors(errors): SerdeJsonErrors) -> Self {
		let errors = errors
			.iter()
			.map(|error| format!("- {error}"))
			.collect::<Vec<_>>()
			.join("\n");
		Self {
			msg: format!("Failed to serialize data:\n{errors}"),
			lvl: Level::Error,
		}
	}
}
