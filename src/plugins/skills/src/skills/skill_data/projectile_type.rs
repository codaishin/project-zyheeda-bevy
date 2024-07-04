use crate::{skills::SkillBehavior, traits::GetStaticSkillBehavior};
use behaviors::components::{Plasma, Projectile};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum ProjectileType {
	Plasma,
}

impl ProjectileType {
	pub(crate) fn projectile_behavior(&self) -> SkillBehavior {
		match self {
			ProjectileType::Plasma => Projectile::<Plasma>::behavior(),
		}
	}
}
