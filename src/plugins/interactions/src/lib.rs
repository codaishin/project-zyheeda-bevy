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
	traits::{
		delta::Delta,
		handles_destruction::HandlesDestruction,
		handles_interactions::{BeamParameters, HandlesInteractions},
		handles_life::HandlesLife,
		handles_lifetime::HandlesLifetime,
		thread_safe::ThreadSafe,
	},
};
use components::{
	beam::{Beam, BeamCommand},
	effect::{deal_damage::DealDamageEffect, gravity::GravityEffect},
	gravity_affected::GravityAffected,
	interacting_entities::InteractingEntities,
	interactions::Interactions,
	is::{Fragile, InterruptableRay, Is},
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

impl<TLifeCyclePlugin> InteractionsPlugin<TLifeCyclePlugin>
where
	TLifeCyclePlugin: ThreadSafe + HandlesDestruction + HandlesLifetime + HandlesLife,
{
	pub fn from_plugin(_: &TLifeCyclePlugin) -> Self {
		Self(PhantomData)
	}
}

impl<TLifeCyclePlugin> Plugin for InteractionsPlugin<TLifeCyclePlugin>
where
	TLifeCyclePlugin: ThreadSafe + HandlesDestruction + HandlesLifetime + HandlesLife,
{
	fn build(&self, app: &mut App) {
		app
			// Deal health damage
			.register_required_components::<DealDamageEffect, InteractingEntities>()
			.add_observer(DealDamageEffect::update_blockers_observer)
			.add_interaction::<DealDamageEffect, TLifeCyclePlugin::TLife>()
			// Apply gravity effect
			.register_required_components::<GravityEffect, InteractingEntities>()
			.add_observer(GravityEffect::update_blockers_observer)
			.add_interaction::<GravityEffect, GravityAffected>()
			.add_systems(Update, Update::delta.pipe(apply_gravity_pull))
			// Apply force effect
			.register_required_components::<ForceEffect, InteractingEntities>()
			.add_observer(ForceEffect::update_blockers_observer)
			.add_interaction::<ForceEffect, ForceAffected>()
			// Apply behaviors
			.add_event::<InteractionEvent>()
			.add_event::<InteractionEvent<Ray>>()
			.init_resource::<TrackInteractionDuplicates>()
			.init_resource::<TrackRayInteractions>()
			.add_systems(Update, apply_fragile_blocks::<TLifeCyclePlugin::TDestroy>)
			.add_systems(
				Update,
				(
					map_collision_events_to::<InteractionEvent, TrackInteractionDuplicates>,
					execute_ray_caster
						.pipe(apply_interruptable_ray_blocks)
						.pipe(map_ray_cast_result_to_interaction_events)
						.pipe(send_interaction_events::<TrackRayInteractions>),
				)
					.chain(),
			)
			.add_systems(Update, update_interacting_entities)
			.add_systems(Update, Beam::execute::<TLifeCyclePlugin>);
	}
}

trait AddInteraction {
	fn add_interaction<TActor, TTarget>(&mut self) -> &mut Self
	where
		TActor: ActOn<TTarget> + Clone + Component<Mutability = Mutable>,
		TTarget: Component<Mutability = Mutable>;
}

impl AddInteraction for App {
	fn add_interaction<TActor, TTarget>(&mut self) -> &mut Self
	where
		TActor: ActOn<TTarget> + Clone + Component<Mutability = Mutable>,
		TTarget: Component<Mutability = Mutable>,
	{
		self
			// require basic interaction tracking
			.register_required_components::<TActor, Interactions<TActor, TTarget>>()
			// apply interactions
			.add_systems(
				Update,
				(
					Update::delta.pipe(TActor::act_on::<TTarget>),
					Interactions::<TActor, TTarget>::untrack_non_interacting_targets,
				)
					.chain(),
			)
	}
}

impl<TDependencies> HandlesInteractions for InteractionsPlugin<TDependencies> {
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
