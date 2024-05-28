pub mod components;
pub mod traits;

pub(crate) mod systems;

use bevy::{
	app::{App, Plugin, Update},
	ecs::{component::Component, schedule::IntoSystemConfigs},
};
use bevy_rapier3d::geometry::CollidingEntities;
use systems::{
	add_colliding_entities::add_colliding_entities,
	apply_gravity::apply_gravity,
	apply_gravity_effect::apply_gravity_effect,
	detect_gravity_effected::detect_gravity_effected,
};
use traits::GetGravityPull;

pub struct GravityPlugin;

impl Plugin for GravityPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Update, apply_gravity);
	}
}

pub trait AddGravityInteraction {
	fn register_gravity_source<TGravitySource: Component + GetGravityPull>(&mut self);
}

impl AddGravityInteraction for App {
	fn register_gravity_source<TGravitySource: Component + GetGravityPull>(&mut self) {
		self.add_systems(
			Update,
			(
				detect_gravity_effected::<CollidingEntities, TGravitySource>,
				add_colliding_entities::<TGravitySource>,
				apply_gravity_effect::<CollidingEntities, TGravitySource>,
			)
				.chain(),
		);
	}
}
