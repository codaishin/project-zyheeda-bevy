pub mod components;
pub mod events;
pub mod traits;

mod resources;
mod systems;

use bevy::{
	app::{App, Plugin},
	ecs::{component::Component, schedule::IntoSystemConfigs},
	time::Virtual,
};
use bevy_rapier3d::plugin::RapierContext;
use common::{components::Health, labels::Labels};
use components::{blocker::BlockerInsertCommand, DealsDamage};
use events::{InteractionEvent, Ray};
use resources::{
	track_interaction_duplicates::TrackInteractionDuplicates,
	track_ray_interactions::TrackRayInteractions,
};
use systems::{
	destroy::destroy,
	destroy_dead::set_dead_to_be_destroyed,
	interactions::{
		apply_fragile_blocks::apply_fragile_blocks,
		collision::collision_interaction,
		delay::delay,
		interacting_entities::interacting_entities,
		map_collision_events::map_collision_events,
		send_flushed_interactions::send_flushed_interactions,
	},
	ray_cast::{
		apply_interruptable_blocks::apply_interruptable_ray_blocks,
		execute_ray_caster::execute_ray_caster,
		map_ray_cast_results_to_interaction_event::map_ray_cast_result_to_interaction_changes,
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
			.add_systems(Labels::PROCESSING, set_dead_to_be_destroyed)
			.add_systems(Labels::PROCESSING, BlockerInsertCommand::system)
			.add_systems(Labels::PROCESSING, apply_interruptable_ray_blocks)
			.add_systems(Labels::PROCESSING, apply_fragile_blocks)
			.add_systems(
				Labels::PROPAGATION,
				(
					map_collision_events::<InteractionEvent, TrackInteractionDuplicates>,
					map_ray_cast_result_to_interaction_changes::<TrackRayInteractions>,
					send_flushed_interactions::<TrackRayInteractions>,
					interacting_entities,
				)
					.chain(),
			)
			.add_systems(Labels::PROPAGATION, destroy)
			.add_systems(Labels::PROPAGATION, execute_ray_caster::<RapierContext>);
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
		self.add_systems(
			Labels::PROCESSING,
			(
				collision_interaction::<TActor, TTarget>,
				delay::<TActor, TTarget, Virtual>,
			)
				.chain(),
		)
	}
}
