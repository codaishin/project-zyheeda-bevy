use crate::traits::MovementData;
use bevy::{
	ecs::{
		component::Component,
		entity::Entity,
		query::With,
		system::{Commands, Query},
	},
	transform::components::Transform,
};
use bevy_rapier3d::dynamics::Velocity;
use common::components::{Player, VoidSphere};

pub(crate) fn void_sphere_behavior<TMovementConfig: Component + MovementData>(
	mut commands: Commands,
	void_spheres: Query<(Entity, &Transform, &TMovementConfig), With<VoidSphere>>,
	players: Query<&Transform, With<Player>>,
) {
	let Ok(player_transform) = players.get_single() else {
		return;
	};
	let target = player_transform.translation;

	for (id, transform, config) in &void_spheres {
		let (speed, ..) = config.get_movement_data();
		let position = transform.translation;
		commands.entity(id).insert(Velocity::linear(
			(target - position).normalize() * speed.to_f32(),
		));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::MovementMode, traits::MovementData};
	use bevy::{
		app::{App, Update},
		math::Vec3,
		transform::components::Transform,
	};
	use bevy_rapier3d::dynamics::Velocity;
	use common::tools::UnitsPerSecond;

	#[derive(Component)]
	struct _MovementConfig(f32);

	impl MovementData for _MovementConfig {
		fn get_movement_data(&self) -> (UnitsPerSecond, MovementMode) {
			(UnitsPerSecond::new(self.0), MovementMode::Fast)
		}
	}

	fn setup(player_position: Vec3) -> App {
		let mut app = App::new();
		app.add_systems(Update, void_sphere_behavior::<_MovementConfig>);
		app.world
			.spawn((Transform::from_translation(player_position), Player));

		app
	}

	#[test]
	fn velocity_to_follow_player() {
		let player_position = Vec3::new(1., 2., 3.);
		let mut app = setup(player_position);
		let void_sphere = app
			.world
			.spawn((Transform::default(), VoidSphere, _MovementConfig(42.)))
			.id();

		app.update();

		let void_sphere = app.world.entity(void_sphere);

		assert_eq!(
			Some(&Velocity::linear(player_position.normalize() * 42.)),
			void_sphere.get::<Velocity>()
		);
	}

	#[test]
	fn velocity_to_follow_player_from_offset() {
		let player_position = Vec3::new(1., 2., 3.);
		let mut app = setup(player_position);
		let position = Vec3::new(4., 5., 6.);
		let void_sphere = app
			.world
			.spawn((
				Transform::from_translation(position),
				VoidSphere,
				_MovementConfig(42.),
			))
			.id();

		app.update();

		let void_sphere = app.world.entity(void_sphere);

		assert_eq!(
			Some(&Velocity::linear(
				(player_position - position).normalize() * 42.
			)),
			void_sphere.get::<Velocity>()
		);
	}
}
