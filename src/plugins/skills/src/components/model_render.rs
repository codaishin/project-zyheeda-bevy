use common::components::AssetModel;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub enum ModelRender {
	#[default]
	None,
	Hand(AssetModel),
	Forearm(AssetModel),
}
