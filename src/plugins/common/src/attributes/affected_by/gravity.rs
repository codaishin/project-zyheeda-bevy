use super::AffectedBy;
use crate::{effects::gravity::Gravity, traits::handles_effects::HandlesEffect};
use bevy::prelude::*;

impl AffectedBy<Gravity> {
	pub fn bundle_via<TPlugin>(self) -> impl Bundle
	where
		TPlugin: HandlesEffect<Gravity>,
	{
		TPlugin::attribute(self)
	}
}
