use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum ItemType {
	#[default]
	Pistol,
	Bracer,
	ForceEssence,
}
