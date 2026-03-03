use crate::square_grid_graph::context::{DividedToZero, MultipliedTooHigh};
use common::errors::{ErrorData, Level};
use std::fmt::Display;

pub trait ToSubdivided: Sized {
	fn to_subdivided(&self, subdivisions: u8) -> Result<Self, SubdivisionError>;
}

#[derive(Debug, PartialEq)]
pub enum SubdivisionError {
	CannotSubdivide,
	CellDistanceZero(DividedToZero),
	CellCountMaxedOut(MultipliedTooHigh),
}

impl Display for SubdivisionError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			SubdivisionError::CannotSubdivide => write!(f, "cannot subdivide grid"),
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
