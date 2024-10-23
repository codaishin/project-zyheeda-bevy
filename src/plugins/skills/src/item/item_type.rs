use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum SkillItemType {
	#[default]
	Pistol,
	Bracer,
}
