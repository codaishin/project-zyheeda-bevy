use bevy::prelude::*;
use common::errors::{ErrorData, Level};
use std::{
	collections::{HashMap, HashSet},
	fmt::Display,
};
use zyheeda_core::prelude::*;

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

impl<TError> Display for ContextIOError<TError>
where
	TError: Display,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ContextIOError::FileError(error) => write!(f, "{error}"),
			ContextIOError::SerdeErrors(errors) => write!(f, "{errors}"),
			ContextIOError::LockPoisoned(error) => write!(f, "{error}"),
		}
	}
}

impl<TError> ErrorData for ContextIOError<TError>
where
	TError: Display,
{
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl Display {
		"IO error"
	}

	fn into_details(self) -> impl Display {
		self
	}
}

#[derive(Debug)]
pub(crate) enum SerializationOrLockError {
	SerializationErrors(SerializationErrors),
	LockPoisoned(LockPoisonedError),
}

impl Display for SerializationOrLockError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			SerializationOrLockError::SerializationErrors(errors) => {
				write!(f, "{errors}")
			}
			SerializationOrLockError::LockPoisoned(error) => {
				write!(f, "{error}")
			}
		}
	}
}

impl ErrorData for SerializationOrLockError {
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl Display {
		"Serialization failed"
	}

	fn into_details(self) -> impl Display {
		self
	}
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub(crate) enum DeserializationOrLockError<TNoInsert> {
	DeserializationErrors(IOErrors<InsertionError<TNoInsert>, Load>),
	LockPoisoned(LockPoisonedError),
}

impl<TNoInsert> Display for DeserializationOrLockError<TNoInsert>
where
	TNoInsert: Display,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			DeserializationOrLockError::DeserializationErrors(errors) => write!(f, "{errors}"),
			DeserializationOrLockError::LockPoisoned(error) => {
				write!(f, "{error}")
			}
		}
	}
}

impl<TNoInsert> ErrorData for DeserializationOrLockError<TNoInsert>
where
	TNoInsert: Display,
{
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl Display {
		"Deserialization failed"
	}

	fn into_details(self) -> impl Display {
		self
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
			InsertionError::CouldNotInsert(error) => write!(f, "Failed Insertion: {error}"),
			InsertionError::UnknownComponents(components) => {
				write_iter!(f, "UnknownComponents: ", components)
			}
		}
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct LockPoisonedError;

impl Display for LockPoisonedError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "lock poisoned")
	}
}

impl ErrorData for LockPoisonedError {
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl Display {
		"Lock was poisoned"
	}

	fn into_details(self) -> impl Display {
		self
	}
}

#[derive(Debug)]
pub struct SerializationErrors(pub(crate) HashMap<Entity, EntitySerializationErrors>);

impl SerializationErrors {
	fn iter(&self) -> impl Iterator<Item = SerializationErrorsItem<'_>> {
		self.0
			.iter()
			.map(|(entity, errors)| SerializationErrorsItem { entity, errors })
	}
}

impl Display for SerializationErrors {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write_iter!(f, self)
	}
}

struct SerializationErrorsItem<'a> {
	entity: &'a Entity,
	errors: &'a EntitySerializationErrors,
}

impl Display for SerializationErrorsItem<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "({}: {})", self.entity, self.errors)
	}
}

#[derive(Debug)]
pub(crate) struct EntitySerializationErrors(pub(crate) Vec<SerdeJsonError>);

impl Display for EntitySerializationErrors {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write_iter!(f, self.0)
	}
}

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

impl<T, TPhase> Display for IOErrors<T, TPhase>
where
	T: Display,
	TPhase: Display,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "IO Operation ({}) failed: ", self.phase)?;
		write_iter!(f, self.items)
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
