pub mod components;
pub mod events;
mod systems;
mod traits;

use bevy::{
	app::{App, Plugin, PostUpdate, Update},
	ecs::{component::Component, schedule::IntoSystemConfigs},
	time::Virtual,
};
use bevy_rapier3d::plugin::RapierContext;
use common::components::Health;
use components::DealsDamage;
use events::{InteractionEvent, Ray};
use systems::{
	destroy::destroy,
	destroy_dead::set_dead_to_be_destroyed,
	interactions::{
		beam_blocked_by::beam_blocked_by,
		collision::collision_interaction,
		collision_start_event_to_interaction_event::collision_start_event_to_interaction_event,
		delay::delay,
		fragile_blocked_by::fragile_blocked_by,
	},
	ray_cast::{
		execute_ray_caster::execute_ray_caster,
		ray_cast_result_to_events::ray_cast_result_to_interaction_events,
	},
};
use traits::ActOn;

pub struct InteractionsPlugin;

impl Plugin for InteractionsPlugin {
	fn build(&self, app: &mut App) {
		app.add_event::<InteractionEvent>()
			.add_event::<InteractionEvent<Ray>>()
			.add_interaction::<DealsDamage, Health>()
			.add_systems(Update, ray_cast_result_to_interaction_events)
			.add_systems(Update, collision_start_event_to_interaction_event)
			.add_systems(Update, set_dead_to_be_destroyed)
			.add_systems(PostUpdate, destroy)
			.add_systems(PostUpdate, execute_ray_caster::<RapierContext>);
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
			Update,
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
			Update,
			(
				beam_blocked_by::<TComponent>,
				fragile_blocked_by::<TComponent>,
			),
		)
	}
}
