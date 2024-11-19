mod plasma_projectile;
mod traits;

use bevy::prelude::ChildBuilder;
use common::traits::prefab::GetOrCreateAssets;
use plasma_projectile::PlasmaProjectile;
use serde::{Deserialize, Serialize};
use traits::ProjectileSubtype;

#[derive(Debug, PartialEq, Default, Clone, Copy, Serialize, Deserialize)]
pub enum SubType {
	#[default]
	Plasma,
}

macro_rules! match_impl {
	($sub_type:expr) => {
		match $sub_type {
			SubType::Plasma => PlasmaProjectile,
		}
	};
}

impl SubType {
	pub fn spawn_contact(&self, parent: &mut ChildBuilder, assets: &mut impl GetOrCreateAssets) {
		match_impl!(self).spawn_contact(parent, assets);
	}

	pub fn spawn_projection(&self, parent: &mut ChildBuilder) {
		match_impl!(self).spawn_projection(parent)
	}
}
