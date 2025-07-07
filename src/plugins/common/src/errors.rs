use bevy::{ecs::error::BevyError, math::InvalidDirectionError, reflect::TypePath};
use std::{
	any::type_name,
	convert::Infallible,
	error::Error as StdError,
	fmt::{Debug, Display},
	io::Error as IoError,
	marker::PhantomData,
};

#[derive(Debug, PartialEq, Clone)]
pub enum Level {
	Warning,
	Error,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Error {
	Single { msg: String, lvl: Level },
	Multiple(Vec<Self>),
}

impl From<InvalidDirectionError> for Error {
	fn from(value: InvalidDirectionError) -> Self {
		match value {
			InvalidDirectionError::Zero => Self::Single {
				msg: "Encountered zero length direction".to_owned(),
				lvl: Level::Error,
			},
			InvalidDirectionError::Infinite => Self::Single {
				msg: "Encountered infinite length direction".to_owned(),
				lvl: Level::Error,
			},
			InvalidDirectionError::NaN => Self::Single {
				msg: "Encountered NaN length direction".to_owned(),
				lvl: Level::Error,
			},
		}
	}
}

impl From<IoError> for Error {
	fn from(value: IoError) -> Self {
		Self::Single {
			msg: value.to_string(),
			lvl: Level::Error,
		}
	}
}

impl From<BevyError> for Error {
	fn from(value: BevyError) -> Self {
		Self::Single {
			msg: value.to_string(),
			lvl: Level::Error,
		}
	}
}

impl From<Infallible> for Error {
	fn from(_: Infallible) -> Self {
		unreachable!("If you managed to get here, I am seriously impressed (not in a good way...)")
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

pub struct UniqueViolation<T> {
	_p: PhantomData<T>,
	found: Found,
}

impl<T> Debug for UniqueViolation<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("UniqueViolation")
			.field("_p", &self._p)
			.field("found", &self.found)
			.finish()
	}
}

impl<T> PartialEq for UniqueViolation<T> {
	fn eq(&self, other: &Self) -> bool {
		self.found == other.found
	}
}

impl UniqueViolation<()> {
	pub fn none_of<T>() -> UniqueViolation<T> {
		UniqueViolation {
			_p: PhantomData,
			found: Found::None,
		}
	}

	pub fn multiple_of<T>() -> UniqueViolation<T> {
		UniqueViolation {
			_p: PhantomData,
			found: Found::Multiple,
		}
	}
}

#[derive(Debug, PartialEq)]
enum Found {
	None,
	Multiple,
}

impl<T> From<UniqueViolation<T>> for Error {
	fn from(UniqueViolation { found, .. }: UniqueViolation<T>) -> Self {
		let found = match found {
			Found::None => "none",
			Found::Multiple => "multiple",
		};
		Self::Single {
			msg: format!(
				"Found {} {}, when needing one unique",
				found,
				type_name::<T>()
			),
			lvl: Level::Error,
		}
	}
}
