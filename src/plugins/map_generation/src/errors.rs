use common::errors::{ErrorData, Level};
use std::fmt::Display;

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum GridError {
	NoRefToCellDefinition,
	NoCellDefinition,
	GridIndicesWithNoCell(Vec<(u32, u32)>),
}

impl Display for GridError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Faulty grid encountered: {self:?}")
	}
}

impl ErrorData for GridError {
	type TContext = Self;

	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> String {
		"Grid error".to_owned()
	}

	fn context(&self) -> &Self::TContext {
		self
	}
}
