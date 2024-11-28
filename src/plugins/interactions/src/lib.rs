pub mod components;
pub mod events;
pub mod traits;

mod resources;
mod systems;

use bevy::prelude::*;
use bevy_rapier3d::plugin::RapierContext;
use common::{
	self,
	components::Health,
	labels::Labels,
	traits::{
		handles_beams::{BeamParameters, HandlesBeams},
		handles_destruction::HandlesDestruction,
		handles_lifetime::HandlesLifetime,
	},
};
use components::{
	acted_on_targets::ActedOnTargets,
	beam::{Beam, BeamCommand},
	blocker::BlockerInsertCommand,
	deals_damage::DealsDamage,
	effected_by_gravity::EffectedByGravity,
	gravity::Gravity,
	interacting_entities::InteractingEntities,
};
use events::{InteractionEvent, Ray};
use resources::{
	track_interaction_duplicates::TrackInteractionDuplicates,
	track_ray_interactions::TrackRayInteractions,
};
use std::marker::PhantomData;
use systems::{
	gravity_pull::gravity_pull,
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
use traits::ActOn;

pub struct InteractionsPlugin<TLifeCyclePlugin>(PhantomData<TLifeCyclePlugin>);

impl<TLifeCyclePlugin> InteractionsPlugin<TLifeCyclePlugin>
where
	TLifeCyclePlugin: Plugin + HandlesDestruction + HandlesLifetime,
{
	pub fn depends_on(_: &TLifeCyclePlugin) -> Self {
		Self(PhantomData)
	}
}

impl<TLifeCyclePlugin> Plugin for InteractionsPlugin<TLifeCyclePlugin>
where
	TLifeCyclePlugin: Plugin + HandlesDestruction + HandlesLifetime,
{
	fn build(&self, app: &mut App) {
		let processing_label = Labels::PROCESSING.label();
		let processing_delta = Labels::PROCESSING.delta();

		app.add_event::<InteractionEvent>()
			.add_event::<InteractionEvent<Ray>>()
			.init_resource::<TrackInteractionDuplicates>()
			.init_resource::<TrackRayInteractions>()
			.add_interaction::<DealsDamage, Health>()
			.add_interaction::<Gravity, EffectedByGravity>()
			.add_systems(processing_label.clone(), BlockerInsertCommand::system)
			.add_systems(
				processing_label.clone(),
				apply_fragile_blocks::<TLifeCyclePlugin::TDestroy>,
			)
			.add_systems(
				processing_label.clone(),
				processing_delta.pipe(gravity_pull),
			)
			.add_systems(
				processing_label.clone(),
				(
					map_collision_events_to::<InteractionEvent, TrackInteractionDuplicates>,
					execute_ray_caster::<RapierContext>
						.pipe(apply_interruptable_ray_blocks)
						.pipe(map_ray_cast_result_to_interaction_events)
						.pipe(send_interaction_events::<TrackRayInteractions>),
				)
					.chain(),
			)
			.add_systems(Labels::PROPAGATION.label(), update_interacting_entities)
			.add_systems(Update, Beam::execute::<TLifeCyclePlugin::TLifetime>);
	}
}

trait AddInteraction {
	fn add_interaction<TActor: ActOn<TTarget> + Clone + Component, TTarget: Component>(
		&mut self,
	) -> &mut Self;
}

impl AddInteraction for App {
	fn add_interaction<TActor: ActOn<TTarget> + Clone + Component, TTarget: Component>(
		&mut self,
	) -> &mut Self {
		let label = Labels::PROPAGATION.label();
		let delta = Labels::PROPAGATION.delta();

		self.add_systems(
			label,
			(
				add_component_to::<TActor, InteractingEntities>,
				add_component_to::<TActor, ActedOnTargets<TActor>>,
				delta.pipe(act_interaction::<TActor, TTarget>),
				untrack_non_interacting_targets::<TActor>,
			)
				.chain(),
		)
	}
}

impl<TLifeCyclePlugin> HandlesBeams for InteractionsPlugin<TLifeCyclePlugin> {
	fn beam_from<T>(value: &T) -> impl Bundle
	where
		T: BeamParameters,
	{
		BeamCommand::from(value)
	}
}
