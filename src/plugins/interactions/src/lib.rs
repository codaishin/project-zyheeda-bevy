pub mod components;
mod events;
mod systems;
mod traits;

use bevy::{
	app::{App, Plugin, PostUpdate, Update},
	ecs::component::Component,
};
use common::components::Health;
use components::DealsDamage;
use events::RayCastEvent;
use systems::{
	destroy::destroy,
	destroy_dead::set_dead_to_be_destroyed,
	interactions::{collision::collision_interaction, ray_cast::ray_cast_interaction},
};
use traits::ActOn;

pub struct InteractionsPlugin;

impl Plugin for InteractionsPlugin {
	fn build(&self, app: &mut App) {
		app.add_event::<RayCastEvent>()
			.add_interaction::<DealsDamage, Health>()
			.add_systems(Update, set_dead_to_be_destroyed)
			.add_systems(PostUpdate, destroy);
	}
}

trait AddInteraction {
	fn add_interaction<TActor: ActOn<TTarget> + Component, TTarget: Component>(
		&mut self,
	) -> &mut Self;
}

impl AddInteraction for App {
	fn add_interaction<TActor: ActOn<TTarget> + Component, TTarget: Component>(
		&mut self,
	) -> &mut Self {
		self.add_systems(
			Update,
			(
				collision_interaction::<TActor, TTarget>,
				ray_cast_interaction::<TActor, TTarget>,
			),
		)
	}
}
