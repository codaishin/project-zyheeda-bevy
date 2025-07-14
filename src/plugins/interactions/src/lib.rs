pub mod components;
pub mod events;
pub mod traits;

mod observers;
mod resources;
mod systems;

use crate::{
	components::{effect::force::ForceEffect, force_affected::ForceAffected},
	observers::update_blockers::UpdateBlockersObserver,
	systems::interactions::act_on::ActOnSystem,
};
use bevy::{ecs::component::Mutable, prelude::*};
use common::{
	self,
	blocker::Blocker,
	components::life::Life,
	traits::{
		delta::Delta,
		handles_interactions::{BeamParameters, HandlesInteractions},
		handles_saving::{HandlesSaving, SavableComponent},
		thread_safe::ThreadSafe,
	},
};
use components::{
	beam::{Beam, BeamCommand},
	effect::{deal_damage::DealDamageEffect, gravity::GravityEffect},
	gravity_affected::GravityAffected,
	interacting_entities::InteractingEntities,
	is::{Fragile, InterruptableRay, Is},
	running_interactions::RunningInteractions,
};
use events::{InteractionEvent, Ray};
use resources::{
	track_interaction_duplicates::TrackInteractionDuplicates,
	track_ray_interactions::TrackRayInteractions,
};
use std::marker::PhantomData;
use systems::{
	gravity_affected::apply_gravity_pull,
	interactions::{
		apply_fragile_blocks::apply_fragile_blocks,
		map_collision_events::map_collision_events_to,
		update_interacting_entities::update_interacting_entities,
	},
	ray_cast::{
		apply_interruptable_blocks::apply_interruptable_ray_blocks,
		execute_ray_caster::execute_ray_caster,
		map_ray_cast_results_to_interaction_event::map_ray_cast_result_to_interaction_events,
		send_interaction_events::send_interaction_events,
	},
};
use traits::act_on::ActOn;

pub struct InteractionsPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TSaveGame> InteractionsPlugin<TSaveGame>
where
	TSaveGame: ThreadSafe + HandlesSaving,
{
	pub fn from_plugin(_: &TSaveGame) -> Self {
		Self(PhantomData)
	}
}

impl<TSaveGame> Plugin for InteractionsPlugin<TSaveGame>
where
	TSaveGame: ThreadSafe + HandlesSaving,
{
	fn build(&self, app: &mut App) {
		app
			// Deal health damage
			.register_required_components::<DealDamageEffect, InteractingEntities>()
			.add_observer(DealDamageEffect::update_blockers_observer)
			.add_interaction::<DealDamageEffect, Life, TSaveGame>()
			// Apply gravity effect
			.register_required_components::<GravityEffect, InteractingEntities>()
			.add_observer(GravityEffect::update_blockers_observer)
			.add_interaction::<GravityEffect, GravityAffected, TSaveGame>()
			.add_systems(Update, Update::delta.pipe(apply_gravity_pull))
			// Apply force effect
			.register_required_components::<ForceEffect, InteractingEntities>()
			.add_observer(ForceEffect::update_blockers_observer)
			.add_interaction::<ForceEffect, ForceAffected, TSaveGame>()
			// Apply interactions
			.add_event::<InteractionEvent>()
			.add_event::<InteractionEvent<Ray>>()
			.init_resource::<TrackInteractionDuplicates>()
			.init_resource::<TrackRayInteractions>()
			.add_systems(
				Update,
				(
					apply_fragile_blocks,
					Beam::execute,
					execute_ray_caster
						.pipe(apply_interruptable_ray_blocks)
						.pipe(map_ray_cast_result_to_interaction_events)
						.pipe(send_interaction_events::<TrackRayInteractions>),
					map_collision_events_to::<InteractionEvent, TrackInteractionDuplicates>,
					update_interacting_entities, // must be last to ensure `InteractionEvent`s and `InteractingEntities` are synched
				)
					.chain()
					.in_set(CollisionSystems),
			);
	}
}

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct CollisionSystems;

trait AddInteraction {
	fn add_interaction<TActor, TTarget, TSaveGame>(&mut self) -> &mut Self
	where
		TActor: ActOn<TTarget> + Component<Mutability = Mutable> + SavableComponent,
		TTarget: Component<Mutability = Mutable> + SavableComponent,
		TSaveGame: HandlesSaving;
}

impl AddInteraction for App {
	fn add_interaction<TActor, TTarget, TSaveGame>(&mut self) -> &mut Self
	where
		TActor: ActOn<TTarget> + Component<Mutability = Mutable> + SavableComponent,
		TTarget: Component<Mutability = Mutable> + SavableComponent,
		TSaveGame: HandlesSaving,
	{
		TSaveGame::register_savable_component::<TActor>(self);
		TSaveGame::register_savable_component::<TTarget>(self);
		TSaveGame::register_savable_component::<RunningInteractions<TActor, TTarget>>(self);

		self.register_required_components::<TActor, RunningInteractions<TActor, TTarget>>()
			.add_systems(
				Update,
				(
					Update::delta.pipe(TActor::act_on::<TTarget>),
					RunningInteractions::<TActor, TTarget>::untrack_non_interacting_targets,
				)
					.chain()
					.in_set(InteractionSystems)
					.after(CollisionSystems),
			)
	}
}

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone)]
pub struct InteractionSystems;

impl<TDependencies> HandlesInteractions for InteractionsPlugin<TDependencies> {
	type TSystems = InteractionSystems;

	const SYSTEMS: Self::TSystems = InteractionSystems;

	fn is_fragile_when_colliding_with<TBlockers>(blockers: TBlockers) -> impl Bundle
	where
		TBlockers: IntoIterator<Item = Blocker>,
	{
		Is::<Fragile>::interacting_with(blockers)
	}

	fn is_ray_interrupted_by<TBlockers>(blockers: TBlockers) -> impl Bundle
	where
		TBlockers: IntoIterator<Item = Blocker>,
	{
		Is::<InterruptableRay>::interacting_with(blockers)
	}

	fn beam_from<T>(value: &T) -> impl Bundle
	where
		T: BeamParameters,
	{
		BeamCommand::from(value)
	}
}
