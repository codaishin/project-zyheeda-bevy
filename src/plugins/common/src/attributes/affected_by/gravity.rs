use super::AffectedBy;
use crate::{effects::gravity::Gravity, traits::handles_physics::HandlesPhysicalEffect};
use bevy::prelude::*;

impl AffectedBy<Gravity> {
	pub fn component<TPlugin>(self) -> impl Bundle
	where
		TPlugin: HandlesPhysicalEffect<Gravity>,
	{
		TPlugin::into_affected_component(self)
	}
}
