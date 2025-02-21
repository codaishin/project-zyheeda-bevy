use super::Effect;
use crate::InteractionsPlugin;
use bevy::{ecs::system::EntityCommands, prelude::*};
use common::{
	blocker::Blocker,
	effects::force_shield::ForceShield,
	errors::Error,
	traits::{handles_effect::HandlesEffect, prefab::Prefab},
};

impl<TPrefabs, TLifecyclePlugin> HandlesEffect<ForceShield>
	for InteractionsPlugin<(TPrefabs, TLifecyclePlugin)>
{
	type TTarget = ();
	type TEffectComponent = Effect<ForceShield>;

	fn effect(effect: ForceShield) -> Self::TEffectComponent {
		Effect(effect)
	}

	fn attribute(_: Self::TTarget) -> impl Bundle {}
}

impl Prefab<()> for Effect<ForceShield> {
	fn instantiate_on<TAfterInstantiation>(
		&self,
		entity: &mut EntityCommands,
	) -> Result<(), Error> {
		entity.try_insert(Blocker::insert([Blocker::Force]));

		Ok(())
	}
}
