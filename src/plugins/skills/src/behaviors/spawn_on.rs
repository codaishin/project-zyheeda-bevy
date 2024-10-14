use serde::{Deserialize, Serialize};

#[derive(Default, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum SpawnOn {
	#[default]
	Center,
	Slot,
}
