use common::errors::{Error, Level};

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum GridError {
	NoRefToCellDefinition,
	NoCellDefinition,
	GridIndicesWithNoCell(Vec<(usize, usize)>),
}

impl From<GridError> for Error {
	fn from(error: GridError) -> Self {
		Self::Single {
			msg: format!("Faulty grid encountered: {error:?}"),
			lvl: Level::Error,
		}
	}
}

#[derive(Debug, PartialEq)]
pub(crate) struct NoGridGraphSet;

impl From<NoGridGraphSet> for Error {
	fn from(_: NoGridGraphSet) -> Self {
		Self::Single {
			msg: "Grid graph was not set".to_owned(),
			lvl: Level::Error,
		}
	}
}
