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
	add_gravity_effect_collider::add_gravity_effect_collider,
	apply_gravity_effect::apply_gravity_effect,
};
use traits::{GetGravityEffectCollider, GetGravityPull};

pub struct GravityPlugin;

impl Plugin for GravityPlugin {
	fn build(&self, _: &mut App) {}
}

pub trait AddGravityInteraction {
	fn register_gravity_source<TGravitySource>(&mut self)
	where
		TGravitySource: Component + GetGravityPull + GetGravityEffectCollider;
}

impl AddGravityInteraction for App {
	fn register_gravity_source<TGravitySource>(&mut self)
	where
		TGravitySource: Component + GetGravityPull + GetGravityEffectCollider,
	{
		let gravity_systems = (
			add_gravity_effect_collider::<TGravitySource>,
			add_colliding_entities::<TGravitySource>,
			apply_gravity_effect::<CollidingEntities, TGravitySource>,
		);
		self.add_systems(Update, gravity_systems.chain());
	}
}
