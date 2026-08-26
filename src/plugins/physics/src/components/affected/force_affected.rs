use crate::systems::insert_affected::AffectedComponent;
use bevy::prelude::*;
use common::prelude::*;
use macros::{SavableComponent, serde_model};

#[serde_model]
#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Copy)]
#[savable_component(id = "force_affected")]
pub struct ForceAffected(pub(crate) EffectTarget<Force>);

impl From<EffectTarget<Force>> for ForceAffected {
	fn from(target: EffectTarget<Force>) -> Self {
		Self(target)
	}
}

impl AffectedComponent for ForceAffected {
	type TAttribute = EffectTarget<Force>;
}
