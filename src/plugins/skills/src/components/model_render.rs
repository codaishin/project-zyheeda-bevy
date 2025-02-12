use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub enum ModelRender {
	#[default]
	None,
	Hand(String),
	Forearm(String),
}
