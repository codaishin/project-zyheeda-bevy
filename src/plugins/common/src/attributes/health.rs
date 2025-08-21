use crate::{effects::health_damage::HealthDamage, traits::handles_effects::HandlesEffect};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct Health {
	pub current: f32,
	pub max: f32,
}

impl Health {
	pub fn new(value: f32) -> Self {
		Self {
			current: value,
			max: value,
		}
	}

	pub fn bundle_via<TPlugin>(self) -> impl Bundle
	where
		TPlugin: HandlesEffect<HealthDamage>,
	{
		TPlugin::attribute(self)
	}
}
