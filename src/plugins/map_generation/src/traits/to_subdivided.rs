use crate::grid_graph::grid_context::{DividedToZero, MultipliedTooHigh};
use common::errors::Error;

pub trait ToSubdivided: Sized {
	fn to_subdivided(&self, subdivisions: u8) -> Result<Self, SubdivisionError>;
}

#[derive(Debug, PartialEq)]
pub enum SubdivisionError {
	CellDistanceZero(DividedToZero),
	CellCountMaxedOut(MultipliedTooHigh),
}

impl From<SubdivisionError> for Error {
	fn from(error: SubdivisionError) -> Self {
		match error {
			SubdivisionError::CellDistanceZero(error) => Error::from(error),
			SubdivisionError::CellCountMaxedOut(error) => Error::from(error),
		}
	}
}
