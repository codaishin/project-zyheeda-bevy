mod plasma_projectile;
mod traits;

use bevy::prelude::ChildBuilder;
use plasma_projectile::PlasmaProjectile;
use prefabs::traits::GetOrCreateAssets;
use serde::{Deserialize, Serialize};
use traits::Spawn;

#[derive(Debug, PartialEq, Default, Clone, Copy, Serialize, Deserialize)]
pub enum SubType {
	#[default]
	Plasma,
}

impl SubType {
	pub fn spawn(&self, parent: &mut ChildBuilder, assets: &mut impl GetOrCreateAssets) {
		match self {
			SubType::Plasma => PlasmaProjectile::spawn(parent, assets),
		};
	}
}
