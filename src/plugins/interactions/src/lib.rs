pub mod components;
pub mod events;
pub mod traits;

mod resources;
mod systems;

use bevy::{
	app::{App, Plugin},
	ecs::{component::Component, schedule::IntoSystemConfigs},
	prelude::IntoSystem,
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
			.add_systems(Labels::PROCESSING, apply_fragile_blocks)
			.add_systems(
				Labels::PROCESSING,
				(
					map_collision_events_to::<InteractionEvent, TrackInteractionDuplicates>,
					execute_ray_caster::<RapierContext>
						.pipe(apply_interruptable_ray_blocks)
						.pipe(map_ray_cast_result_to_interaction_events)
						.pipe(send_interaction_events::<TrackRayInteractions>),
				)
					.chain(),
			)
			.add_systems(Labels::PROPAGATION, update_interacting_entities)
			.add_systems(Labels::PROPAGATION, destroy);
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
