use bevy::{ecs::error::BevyError, math::InvalidDirectionError, reflect::TypePath};
use std::{
	any::type_name,
	convert::Infallible,
	error::Error as StdError,
	fmt::{Debug, Display},
	io::Error as IoError,
	marker::PhantomData,
};
use zyheeda_core::write_iter;

#[derive(Debug, PartialEq, Clone)]
pub enum Level {
	Warning,
	Error,
}

impl ErrorData for InvalidDirectionError {
	type TDetails = Self;

	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> String {
		"Faulty direction".to_owned()
	}

	fn into_details(self) -> Self::TDetails {
		todo!()
	}
}

impl ErrorData for IoError {
	type TDetails = Self;

	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> String {
		"Io error".to_owned()
	}

	fn into_details(self) -> Self::TDetails {
		self
	}
}

impl ErrorData for BevyError {
	type TDetails = Self;

	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> String {
		"Bevy error".to_owned()
	}

	fn into_details(self) -> Self::TDetails {
		self
	}
}

impl ErrorData for Infallible {
	type TDetails = Self;

	fn level(&self) -> Level {
		match *self {}
	}

	fn label() -> String {
		"Infallible".to_owned()
	}

	fn into_details(self) -> Self::TDetails {
		match self {}
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, TypePath)]
pub enum Unreachable {}

impl Display for Unreachable {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{self:?}: If you see this, the universe broke")
	}
}

impl StdError for Unreachable {}

impl ErrorData for Unreachable {
	type TDetails = Self;

	fn level(&self) -> Level {
		match *self {}
	}

	fn label() -> String {
		"Unreachable".to_owned()
	}

	fn into_details(self) -> Self::TDetails {
		match self {}
	}
}

pub struct UniqueViolation<T> {
	_p: PhantomData<T>,
	found: Found,
}

impl<T> UniqueViolation<T> {
	pub fn none() -> Self {
		Self {
			found: Found::None,
			_p: PhantomData,
		}
	}

	pub fn multiple() -> Self {
		Self {
			found: Found::Multiple,
			_p: PhantomData,
		}
	}
}

impl<T> Debug for UniqueViolation<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("UniqueViolation")
			.field("_p", &self._p)
			.field("found", &self.found)
			.finish()
	}
}

impl<T> Display for UniqueViolation<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let found = match self.found {
			Found::None => "none",
			Found::Multiple => "multiple",
		};
		let type_name = type_name::<T>();

		write!(f, "Found {found} {type_name}")
	}
}

impl<T> PartialEq for UniqueViolation<T> {
	fn eq(&self, other: &Self) -> bool {
		self.found == other.found
	}
}

#[derive(Debug, PartialEq)]
enum Found {
	None,
	Multiple,
}

impl<T> ErrorData for UniqueViolation<T> {
	type TDetails = Self;

	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> String {
		"Uniqueness violated".to_owned()
	}

	fn into_details(self) -> Self::TDetails {
		self
	}
}

pub trait ErrorData {
	type TDetails: Display;

	fn level(&self) -> Level;
	fn label() -> String;
	fn into_details(self) -> Self::TDetails;
}

impl<T> ErrorData for Vec<T>
where
	T: ErrorData,
{
	type TDetails = VecErrorDetails<T::TDetails>;

	fn level(&self) -> Level {
		if self.iter().any(|e| e.level() == Level::Error) {
			return Level::Error;
		}

		Level::Warning
	}

	fn label() -> String {
		format!("Multiple errors: {}", T::label())
	}

	fn into_details(self) -> Self::TDetails {
		VecErrorDetails(self.into_iter().map(|e| e.into_details()).collect())
	}
}

pub struct VecErrorDetails<T>(Vec<T>);

impl<T> Display for VecErrorDetails<T>
where
	T: Display,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write_iter!(f, self.0)
	}
}
