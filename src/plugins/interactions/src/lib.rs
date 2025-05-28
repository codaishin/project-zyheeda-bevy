pub mod components;
pub mod events;
pub mod traits;

mod resources;
mod systems;

use bevy::{ecs::component::Mutable, prelude::*};
use common::{
	self,
	blocker::{Blocker, BlockerInsertCommand},
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
	acted_on_targets::ActedOnTargets,
	beam::{Beam, BeamCommand},
	blockers::ApplyBlockerInsertion,
	effect::{deal_damage::DealDamageEffect, gravity::GravityEffect},
	gravity_affected::GravityAffected,
	interacting_entities::InteractingEntities,
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
		act_interaction::act_interaction,
		add_component::add_component_to,
		apply_fragile_blocks::apply_fragile_blocks,
		map_collision_events::map_collision_events_to,
		untrack_non_interacting_targets::untrack_non_interacting_targets,
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
		app.add_event::<InteractionEvent>()
			.add_event::<InteractionEvent<Ray>>()
			.init_resource::<TrackInteractionDuplicates>()
			.init_resource::<TrackRayInteractions>()
			.add_interaction::<DealDamageEffect, TLifeCyclePlugin::TLife>()
			.add_interaction::<GravityEffect, GravityAffected>()
			.add_systems(Update, BlockerInsertCommand::apply)
			.add_systems(Update, apply_fragile_blocks::<TLifeCyclePlugin::TDestroy>)
			.add_systems(Update, Update::delta.pipe(apply_gravity_pull))
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
		self.add_systems(
			Update,
			(
				add_component_to::<TActor, InteractingEntities>,
				add_component_to::<TActor, ActedOnTargets<TActor>>,
				Update::delta.pipe(act_interaction::<TActor, TTarget>),
				untrack_non_interacting_targets::<TActor>,
			)
				.chain(),
		)
	}
}

impl<TDependencies> HandlesInteractions for InteractionsPlugin<TDependencies> {
	fn is_fragile_when_colliding_with(blockers: &[Blocker]) -> impl Bundle {
		Is::<Fragile>::interacting_with(blockers)
	}

	fn is_ray_interrupted_by(blockers: &[Blocker]) -> impl Bundle {
		Is::<InterruptableRay>::interacting_with(blockers)
	}

	fn beam_from<T>(value: &T) -> impl Bundle
	where
		T: BeamParameters,
	{
		BeamCommand::from(value)
	}
}
