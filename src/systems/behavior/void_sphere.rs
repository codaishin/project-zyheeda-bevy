use crate::components::{Player, SimpleMovement, VoidSphere};
use bevy::{
	ecs::{
		entity::Entity,
		query::With,
		system::{Commands, Query},
	},
	transform::components::Transform,
};

pub fn void_sphere_behavior(
	mut commands: Commands,
	void_spheres: Query<Entity, With<VoidSphere>>,
	players: Query<&Transform, With<Player>>,
) {
	let Ok(player_transform) = players.get_single() else {
		return;
	};
	let target = player_transform.translation;

	for void_sphere in &void_spheres {
		commands
			.entity(void_sphere)
			.insert(SimpleMovement { target });
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{Player, SimpleMovement, VoidSphere};
	use bevy::{
		app::{App, Update},
		math::Vec3,
		transform::components::Transform,
	};

	fn setup(player_position: Vec3) -> App {
		let mut app = App::new();
		app.add_systems(Update, void_sphere_behavior);
		app.world.spawn((
			Transform::from_translation(player_position),
			Player::default(),
		));

		app
	}

	#[test]
	fn follow_player() {
		let player_position = Vec3::new(1., 2., 3.);
		let mut app = setup(player_position);
		let void_sphere = app.world.spawn(VoidSphere).id();

		app.update();

		let void_sphere = app.world.entity(void_sphere);

		assert_eq!(
			Some(&SimpleMovement {
				target: player_position
			}),
			void_sphere.get::<SimpleMovement>()
		);
	}

	#[test]
	fn apply_only_to_void_spheres() {
		let mut app = setup(Vec3::default());
		let not_void_sphere = app.world.spawn_empty().id();

		app.update();

		let not_void_sphere = app.world.entity(not_void_sphere);

		assert_eq!(None, not_void_sphere.get::<SimpleMovement>());
	}
}
