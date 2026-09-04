use common::prelude::*;

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone, Copy)]
pub enum AssetLoadPhase {
	#[default]
	Assets,
	Dependencies,
}

impl From<IsProcessing> for AssetLoadPhase {
	fn from(is_processing: IsProcessing) -> Self {
		match is_processing {
			IsProcessing::Assets => Self::Assets,
			IsProcessing::Dependencies => Self::Dependencies,
		}
	}
}
