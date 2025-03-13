use bevy::ecs::entity::Entity;
use common::errors::{Error, Level};
use serde_json::Error as SerdeJsonError;
use std::{collections::HashMap, io::Error as IoError};

#[derive(Debug, PartialEq, Clone)]
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
