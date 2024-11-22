use common::components::AssetModel;
use serde::{Deserialize, Serialize};
use shaders::components::material_override::MaterialOverride;

#[derive(Debug, PartialEq, Default, Clone)]
pub struct Renderer {
	pub model: ModelRender,
	pub arm_shader: MaterialOverride,
}

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub enum ModelRender {
	#[default]
	None,
	Hand(AssetModel),
	Forearm(AssetModel),
}
