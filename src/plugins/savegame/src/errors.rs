use bevy::ecs::entity::Entity;
use common::errors::{Error, Level};
use std::{
	collections::{HashMap, HashSet},
	fmt::Display,
};

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
	SerdeErrors(IOErrors<SerdeJsonError, Save>),
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
pub(crate) enum DeserializationOrLockError<TNoInsert> {
	DeserializationErrors(IOErrors<InsertionError<TNoInsert>, Load>),
	LockPoisoned(LockPoisonedError),
}

impl<TNoInsert> From<DeserializationOrLockError<TNoInsert>> for Error
where
	TNoInsert: Display,
{
	fn from(value: DeserializationOrLockError<TNoInsert>) -> Self {
		match value {
			DeserializationOrLockError::DeserializationErrors(error) => Self::from(error),
			DeserializationOrLockError::LockPoisoned(error) => Self::from(error),
		}
	}
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub(crate) enum InsertionError<TNoInsert> {
	CouldNotInsert(TNoInsert),
	UnknownComponents(HashSet<String>),
}

impl<TNoInsert> Display for InsertionError<TNoInsert>
where
	TNoInsert: Display,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			InsertionError::CouldNotInsert(e) => write!(f, "Failed Insertion: {e}"),
			InsertionError::UnknownComponents(c) => write!(
				f,
				"UnknownComponents: [{}]",
				c.iter().map(String::as_str).collect::<Vec<_>>().join(", ")
			),
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
pub(crate) struct IOErrors<T, TPhase> {
	pub(crate) items: Vec<T>,
	phase: TPhase,
}

impl<T, TPhase> From<Vec<T>> for IOErrors<T, TPhase>
where
	TPhase: Default,
{
	fn from(errors: Vec<T>) -> Self {
		Self {
			items: errors,
			phase: TPhase::default(),
		}
	}
}

impl<T, TPhase> From<IOErrors<T, TPhase>> for Error
where
	T: Display,
	TPhase: Display,
{
	fn from(IOErrors { items, phase }: IOErrors<T, TPhase>) -> Self {
		let errors = items
			.iter()
			.map(T::to_string)
			.collect::<Vec<_>>()
			.join(", ");
		Self::Single {
			msg: format!("IO Operation ({phase}) failed: [{errors}]"),
			lvl: Level::Error,
		}
	}
}

#[derive(Debug, PartialEq, Default)]
pub(crate) struct Save;

impl Display for Save {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Save")
	}
}

#[derive(Debug, PartialEq, Default)]
pub(crate) struct Load;

impl Display for Load {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Load")
	}
}
