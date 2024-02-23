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
use events::RayCastEvent;
use systems::{
	destroy::destroy,
	destroy_dead::set_dead_to_be_destroyed,
	destroy_fragile::destroy_fragile,
	execute_ray_caster::execute_ray_caster,
	interactions::{
		collision::collision_interaction,
		delay::delay,
		ray_cast::ray_cast_interaction,
	},
};
use traits::ActOn;

pub struct InteractionsPlugin;

impl Plugin for InteractionsPlugin {
	fn build(&self, app: &mut App) {
		app.add_event::<RayCastEvent>()
			.add_interaction::<DealsDamage, Health>()
			.add_systems(Update, execute_ray_caster::<RapierContext>)
			.add_systems(Update, set_dead_to_be_destroyed)
			.add_systems(PostUpdate, (destroy_fragile, destroy).chain());
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
				ray_cast_interaction::<TActor, TTarget>,
				delay::<TActor, TTarget, Virtual>,
			)
				.chain(),
		)
	}
}
