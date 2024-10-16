use serde::{Deserialize, Serialize};

#[derive(Default, PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SpawnOn {
	#[default]
	Center,
	Slot,
}
