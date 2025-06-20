use crate::grid_graph::grid_context::GridDefinitionError;
use common::errors::{Error, Level};

#[derive(Debug, PartialEq)]
pub(crate) enum InsertGraphError {
	GridDefinitionError(GridDefinitionError),
	MapAssetNotFound,
}

impl From<InsertGraphError> for Error {
	fn from(error: InsertGraphError) -> Self {
		match error {
			InsertGraphError::GridDefinitionError(error) => Error::from(error),
			InsertGraphError::MapAssetNotFound => Error {
				msg: "Map asset not found".to_owned(),
				lvl: Level::Error,
			},
		}
	}
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum GridError {
	NoRefToCellDefinition,
	NoCellDefinition,
	NoValidMap,
	GridIndexHasNoCell { x: usize, z: usize },
}

impl From<GridError> for Error {
	fn from(error: GridError) -> Self {
		Self {
			msg: format!("Faulty grid encountered: {:?}", error),
			lvl: Level::Error,
		}
	}
}

#[derive(Debug, PartialEq)]
pub(crate) struct NoGridGraphSet;

impl From<NoGridGraphSet> for Error {
	fn from(_: NoGridGraphSet) -> Self {
		Self {
			msg: "Grid graph was not set".to_owned(),
			lvl: Level::Error,
		}
	}
}
