use crate::{components::Chase, traits::MovementData};
use bevy::{
	ecs::{
		component::Component,
		entity::Entity,
		removal_detection::RemovedComponents,
		system::{Commands, Query},
	},
	transform::components::GlobalTransform,
};
use bevy_rapier3d::dynamics::Velocity;

pub(crate) fn chase<TMovementConfig: Component + MovementData>(
	mut commands: Commands,
	chasers: Query<(Entity, &GlobalTransform, &TMovementConfig, &Chase)>,
	mut removed_chasers: RemovedComponents<Chase>,
	transforms: Query<&GlobalTransform>,
) {
	let chasers_with_valid_target = chasers.iter().filter_map(|(id, transform, conf, chase)| {
		let target = transforms.get(chase.0).ok()?;
		Some((id, transform, conf, target.translation()))
	});

	for id in removed_chasers.read() {
		let Some(mut entity) = commands.get_entity(id) else {
			continue;
		};
		entity.insert(Velocity::zero());
	}

	for (id, transform, conf, target) in chasers_with_valid_target {
		let (speed, ..) = conf.get_movement_data();
		let position = transform.translation();
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
		transform::components::GlobalTransform,
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

	fn setup(foe_position: Vec3) -> (App, Entity) {
		let mut app = App::new();
		app.add_systems(Update, chase::<_MovementConfig>);
		let foe = app
			.world
			.spawn(GlobalTransform::from_translation(foe_position))
			.id();

		(app, foe)
	}

	#[test]
	fn velocity_to_follow_player() {
		let foe_position = Vec3::new(1., 2., 3.);
		let (mut app, foe) = setup(foe_position);
		let chaser = app
			.world
			.spawn((GlobalTransform::default(), Chase(foe), _MovementConfig(42.)))
			.id();

		app.update();

		let chaser = app.world.entity(chaser);

		assert_eq!(
			Some(&Velocity::linear(foe_position.normalize() * 42.)),
			chaser.get::<Velocity>()
		);
	}

	#[test]
	fn velocity_to_follow_player_from_offset() {
		let foe_position = Vec3::new(1., 2., 3.);
		let (mut app, foe) = setup(foe_position);
		let position = Vec3::new(4., 5., 6.);
		let chaser = app
			.world
			.spawn((
				GlobalTransform::from_translation(position),
				Chase(foe),
				_MovementConfig(42.),
			))
			.id();

		app.update();

		let chaser = app.world.entity(chaser);

		assert_eq!(
			Some(&Velocity::linear(
				(foe_position - position).normalize() * 42.
			)),
			chaser.get::<Velocity>()
		);
	}

	#[test]
	fn set_velocity_zero_when_chase_removed() {
		let foe_position = Vec3::new(1., 2., 3.);
		let (mut app, foe) = setup(foe_position);
		let chaser = app
			.world
			.spawn((GlobalTransform::default(), Chase(foe), _MovementConfig(42.)))
			.id();

		app.update();

		app.world.entity_mut(chaser).remove::<Chase>();

		app.update();

		let chaser = app.world.entity(chaser);

		assert_eq!(Some(&Velocity::zero()), chaser.get::<Velocity>());
	}
}
