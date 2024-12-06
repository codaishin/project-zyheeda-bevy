use crate::components::Chase;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::{
	components::Immobilized,
	traits::{
		accessors::get::GetterRef,
		handles_behaviors::ConstantMovementSpeed,
		try_insert_on::TryInsertOn,
	},
};
use std::ops::Deref;

impl<T> ChaseSystem for T {}

pub trait ChaseSystem {
	fn chase(
		mut commands: Commands,
		chasers: Query<(Entity, &GlobalTransform, &Self, &Chase), Without<Immobilized>>,
		mut removed_chasers: RemovedComponents<Chase>,
		transforms: Query<&GlobalTransform>,
	) where
		Self: GetterRef<ConstantMovementSpeed> + Component + Sized,
	{
		let chasers_with_valid_target =
			chasers.iter().filter_map(|(id, transform, conf, chase)| {
				let target = transforms.get(chase.0).ok()?;
				Some((id, transform, conf, target.translation()))
			});

		for id in removed_chasers.read() {
			commands.try_insert_on(id, Velocity::zero());
		}

		for (id, transform, conf, target) in chasers_with_valid_target {
			let ConstantMovementSpeed(speed) = conf.get();
			let position = transform.translation();
			commands.try_insert_on(
				id,
				Velocity::linear((target - position).normalize() * *speed.deref()),
			);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy_rapier3d::dynamics::Velocity;
	use common::{tools::UnitsPerSecond, traits::clamp_zero_positive::ClampZeroPositive};

	#[derive(Component)]
	struct _Enemy(ConstantMovementSpeed);

	impl GetterRef<ConstantMovementSpeed> for _Enemy {
		fn get(&self) -> &ConstantMovementSpeed {
			let _Enemy(speed) = self;
			speed
		}
	}

	fn setup(foe_position: Vec3) -> (App, Entity) {
		let mut app = App::new();
		app.add_systems(Update, _Enemy::chase);
		let foe = app
			.world_mut()
			.spawn(GlobalTransform::from_translation(foe_position))
			.id();

		(app, foe)
	}

	#[test]
	fn velocity_to_follow_player() {
		let foe_position = Vec3::new(1., 2., 3.);
		let (mut app, foe) = setup(foe_position);
		let chaser = app
			.world_mut()
			.spawn((
				GlobalTransform::default(),
				Chase(foe),
				_Enemy(ConstantMovementSpeed(UnitsPerSecond::new(42.))),
			))
			.id();

		app.update();

		let chaser = app.world().entity(chaser);

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
			.world_mut()
			.spawn((
				GlobalTransform::from_translation(position),
				Chase(foe),
				_Enemy(ConstantMovementSpeed(UnitsPerSecond::new(42.))),
			))
			.id();

		app.update();

		let chaser = app.world().entity(chaser);

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
			.world_mut()
			.spawn((
				GlobalTransform::default(),
				Chase(foe),
				_Enemy(ConstantMovementSpeed(UnitsPerSecond::new(42.))),
			))
			.id();

		app.update();

		app.world_mut().entity_mut(chaser).remove::<Chase>();

		app.update();

		let chaser = app.world().entity(chaser);

		assert_eq!(Some(&Velocity::zero()), chaser.get::<Velocity>());
	}

	#[test]
	fn no_velocity_to_follow_player_when_immobilized() {
		let foe_position = Vec3::new(1., 2., 3.);
		let (mut app, foe) = setup(foe_position);
		let chaser = app
			.world_mut()
			.spawn((
				GlobalTransform::default(),
				Chase(foe),
				_Enemy(ConstantMovementSpeed(UnitsPerSecond::new(42.))),
				Immobilized,
			))
			.id();

		app.update();

		let chaser = app.world().entity(chaser);

		assert_eq!(None, chaser.get::<Velocity>());
	}
}
