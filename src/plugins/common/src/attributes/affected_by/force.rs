use super::AffectedBy;
use crate::{effects::force::Force, traits::handles_effects::HandlesEffect};
use bevy::prelude::*;

impl AffectedBy<Force> {
	pub fn bundle_via<TPlugin>(self) -> impl Bundle
	where
		TPlugin: HandlesEffect<Force>,
	{
		TPlugin::attribute(self)
	}
}
