pub mod components;
pub mod events;
pub mod traits;

mod resources;
mod systems;

use bevy::{
	app::{App, Plugin},
	ecs::{component::Component, schedule::IntoSystemConfigs},
	prelude::IntoSystem,
};
use bevy_rapier3d::plugin::RapierContext;
use common::{components::Health, labels::Labels};
use components::{
	acted_on_targets::ActedOnTargets,
	blocker::BlockerInsertCommand,
	deals_damage::DealsDamage,
	interacting_entities::InteractingEntities,
};
use events::{InteractionEvent, Ray};
use resources::{
	track_interaction_duplicates::TrackInteractionDuplicates,
	track_ray_interactions::TrackRayInteractions,
};
use systems::{
	destroy::destroy,
	destroy_dead::set_dead_to_be_destroyed,
	gravity_pull::gravity_pull,
	interactions::{
		act_on_interaction::act_on_interaction,
		add_component::add_component_to,
		apply_fragile_blocks::apply_fragile_blocks,
		delay::delay,
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

pub struct InteractionsPlugin;

impl Plugin for InteractionsPlugin {
	fn build(&self, app: &mut App) {
		app.add_event::<InteractionEvent>()
			.add_event::<InteractionEvent<Ray>>()
			.init_resource::<TrackInteractionDuplicates>()
			.init_resource::<TrackRayInteractions>()
			.add_interaction::<DealsDamage, Health>()
			.add_systems(Labels::PROCESSING.label(), set_dead_to_be_destroyed)
			.add_systems(Labels::PROCESSING.label(), BlockerInsertCommand::system)
			.add_systems(Labels::PROCESSING.label(), apply_fragile_blocks)
			.add_systems(Labels::PROCESSING.label(), gravity_pull)
			.add_systems(
				Labels::PROCESSING.label(),
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
			.add_systems(Labels::PROPAGATION.label(), destroy);
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
				delta.pipe(act_on_interaction::<TActor, TTarget>),
				untrack_non_interacting_targets::<TActor>,
				delta.pipe(delay::<TActor, TTarget>),
			)
				.chain(),
		)
	}
}
