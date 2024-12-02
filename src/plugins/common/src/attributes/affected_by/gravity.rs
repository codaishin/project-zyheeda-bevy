use super::AffectedBy;
use crate::{effects::gravity::Gravity, traits::handles_effect::HandlesEffect};
use bevy::prelude::*;

impl AffectedBy<Gravity> {
	pub fn bundle_via<TPlugin>(self) -> impl Bundle
	where
		TPlugin: HandlesEffect<Gravity, TTarget = AffectedBy<Gravity>>,
	{
		TPlugin::attribute(self)
	}
}
