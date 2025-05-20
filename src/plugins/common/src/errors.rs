use bevy::{ecs::error::BevyError, math::InvalidDirectionError, reflect::TypePath};
use std::{convert::Infallible, error::Error as StdError, fmt::Display, io::Error as IoError};

#[derive(Debug, PartialEq, Clone)]
pub enum Level {
	Warning,
	Error,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Error {
	pub msg: String,
	pub lvl: Level,
}

impl From<InvalidDirectionError> for Error {
	fn from(value: InvalidDirectionError) -> Self {
		match value {
			InvalidDirectionError::Zero => Self {
				msg: "Encountered zero length direction".to_owned(),
				lvl: Level::Error,
			},
			InvalidDirectionError::Infinite => Self {
				msg: "Encountered infinite length direction".to_owned(),
				lvl: Level::Error,
			},
			InvalidDirectionError::NaN => Self {
				msg: "Encountered NaN length direction".to_owned(),
				lvl: Level::Error,
			},
		}
	}
}

impl From<IoError> for Error {
	fn from(value: IoError) -> Self {
		Self {
			msg: value.to_string(),
			lvl: Level::Error,
		}
	}
}

impl From<BevyError> for Error {
	fn from(value: BevyError) -> Self {
		Self {
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
		write!(f, "{:?}: If you see this, the universe broke", self)
	}
}

impl StdError for Unreachable {}
