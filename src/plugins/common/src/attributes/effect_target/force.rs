use super::EffectTarget;
use crate::{effects::force::Force, traits::handles_physics::HandlesPhysicalEffect};
use bevy::prelude::*;

impl EffectTarget<Force> {
	pub fn component<TPlugin>(self) -> impl Bundle
	where
		TPlugin: HandlesPhysicalEffect<Force>,
	{
		TPlugin::into_affected_component(self)
	}
}
