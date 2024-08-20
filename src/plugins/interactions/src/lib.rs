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
use components::DealsDamage;
use events::{InteractionEvent, Ray};
use resources::InteractionsTracker;
use systems::{
	destroy::destroy,
	destroy_dead::set_dead_to_be_destroyed,
	interactions::{
		beam_blocked_by::beam_blocked_by,
		collision::collision_interaction,
		delay::delay,
		fragile_blocked_by::fragile_blocked_by,
		map_collision_events::map_collision_events,
	},
	ray_cast::{
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
			.add_interaction::<DealsDamage, Health>()
			.add_systems(Labels::PROCESSING, set_dead_to_be_destroyed)
			.add_systems(
				Labels::PROPAGATION,
				map_ray_cast_result_to_interaction_changes,
			)
			.add_systems(
				Labels::PROPAGATION,
				map_collision_events::<InteractionEvent, InteractionsTracker>,
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

pub trait RegisterBlocker {
	fn register_blocker<TComponent: Component>(&mut self) -> &mut Self;
}

impl RegisterBlocker for App {
	fn register_blocker<TComponent: Component>(&mut self) -> &mut Self {
		self.add_systems(
			Labels::PROCESSING,
			(
				beam_blocked_by::<TComponent>,
				fragile_blocked_by::<TComponent>,
			),
		)
	}
}
