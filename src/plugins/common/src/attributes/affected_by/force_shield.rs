use super::AffectedBy;
use crate::{effects::force_shield::ForceShield, traits::handles_effect::HandlesEffect};
use bevy::prelude::*;

impl AffectedBy<ForceShield> {
	pub fn bundle_via<TPlugin>(self) -> impl Bundle
	where
		TPlugin: HandlesEffect<ForceShield, TTarget = AffectedBy<ForceShield>>,
	{
		TPlugin::attribute(self)
	}
}
