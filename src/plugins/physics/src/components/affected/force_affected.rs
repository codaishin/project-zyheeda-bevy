use crate::systems::insert_affected::AffectedComponent;
use bevy::prelude::*;
use common::{attributes::effect_target::EffectTarget, effects::force::Force};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[savable_component(id = "force affected")]
pub struct ForceAffected(pub(crate) EffectTarget<Force>);

impl From<EffectTarget<Force>> for ForceAffected {
	fn from(target: EffectTarget<Force>) -> Self {
		Self(target)
	}
}

impl AffectedComponent for ForceAffected {
	type TAttribute = EffectTarget<Force>;
}
