use crate::InteractionsPlugin;
use bevy::prelude::*;
use common::{
	blocker::{Blocker, BlockerInsertCommand},
	effects::force_shield::ForceShield,
	traits::handles_effect::HandlesEffect,
};

#[derive(Component, Debug, PartialEq)]
#[require(BlockerInsertCommand(Self::blockers))]
pub struct ForceShieldEffect(pub(crate) ForceShield);

impl ForceShieldEffect {
	fn blockers() -> BlockerInsertCommand {
		Blocker::insert([Blocker::Force])
	}
}

impl<TLifecyclePlugin> HandlesEffect<ForceShield> for InteractionsPlugin<TLifecyclePlugin> {
	type TTarget = ();
	type TEffectComponent = ForceShieldEffect;

	fn effect(effect: ForceShield) -> Self::TEffectComponent {
		ForceShieldEffect(effect)
	}

	fn attribute(_: Self::TTarget) -> impl Bundle {}
}
