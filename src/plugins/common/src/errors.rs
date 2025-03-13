use bevy::math::InvalidDirectionError;
use std::io::Error as IoError;

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
