use bevy::ecs::entity::Entity;
use common::errors::{Error, Level};
use std::{collections::HashMap, fmt::Display};

#[derive(Debug)]
pub(crate) struct SerdeJsonError(pub(crate) serde_json::Error);

impl Display for SerdeJsonError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

#[cfg(test)]
const SERDE_FIELD_EQUALITIES: &[fn(&serde_json::Error, &serde_json::Error) -> bool] = &[
	|a, b| a.line() == b.line(),
	|a, b| a.column() == b.column(),
	|a, b| a.classify() == b.classify(),
];

#[cfg(test)]
impl PartialEq for SerdeJsonError {
	fn eq(&self, other: &Self) -> bool {
		SERDE_FIELD_EQUALITIES
			.iter()
			.all(|field_equal| field_equal(&self.0, &other.0))
	}
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub(crate) enum ContextIOError<TIOError> {
	FileError(TIOError),
	SerdeErrors(IOErrors<SerdeJsonError>),
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
	DeserializationErrors(IOErrors<InsertionError>),
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

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub(crate) enum InsertionError {
	Serde(SerdeJsonError),
	UnknownComponents(Vec<String>),
}

impl Display for InsertionError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			InsertionError::Serde(e) => write!(f, "Serde Error: {e}"),
			InsertionError::UnknownComponents(c) => write!(f, "UnknownComponents: {c:?}"),
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

#[derive(Debug, PartialEq)]
pub(crate) struct IOErrors<T>(pub(crate) Vec<T>);

impl<T> From<IOErrors<T>> for Error
where
	T: Display,
{
	fn from(IOErrors(errors): IOErrors<T>) -> Self {
		let errors = errors
			.iter()
			.map(|error| format!("- {error}"))
			.collect::<Vec<_>>()
			.join("\n");
		Self::Single {
			msg: format!("IO Operation failed:\n{errors}"),
			lvl: Level::Error,
		}
	}
}
