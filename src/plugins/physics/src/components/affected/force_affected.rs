use bevy::prelude::*;
use common::{
	self,
	attributes::effect_target::EffectTarget,
	effects::force::Force,
	tools::attribute::AttributeOnSpawn,
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

use crate::systems::insert_affected::AffectedComponent;

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct ForceAffected(pub(crate) EffectTarget<Force>);

impl From<AttributeOnSpawn<EffectTarget<Force>>> for ForceAffected {
	fn from(AttributeOnSpawn(target): AttributeOnSpawn<EffectTarget<Force>>) -> Self {
		Self(target)
	}
}

impl AffectedComponent for ForceAffected {
	type TAttribute = EffectTarget<Force>;
}
