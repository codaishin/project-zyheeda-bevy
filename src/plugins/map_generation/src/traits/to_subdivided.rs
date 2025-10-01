use std::fmt::Display;

use common::errors::{ErrorData, Level};

use crate::grid_graph::grid_context::{DividedToZero, MultipliedTooHigh};

pub trait ToSubdivided: Sized {
	fn to_subdivided(&self, subdivisions: u8) -> Result<Self, SubdivisionError>;
}

#[derive(Debug, PartialEq)]
pub enum SubdivisionError {
	CellDistanceZero(DividedToZero),
	CellCountMaxedOut(MultipliedTooHigh),
}

impl Display for SubdivisionError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			SubdivisionError::CellDistanceZero(error) => write!(f, "{error}"),
			SubdivisionError::CellCountMaxedOut(error) => write!(f, "{error}"),
		}
	}
}

impl ErrorData for SubdivisionError {
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl Display {
		"Failed to subdivide"
	}

	fn into_details(self) -> impl Display {
		self
	}
}
