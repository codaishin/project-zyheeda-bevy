use super::InsertIntoEntity;
use crate::{
	behaviors::PlayerBehavior,
	components::{
		marker::{HandGun, Left, Marker, Right, Shoot},
		Side,
		SimpleMovement,
		Skill,
	},
};
use bevy::ecs::system::EntityCommands;

impl InsertIntoEntity for PlayerBehavior {
	fn insert_into_entity(self, entity: &mut EntityCommands) {
		match self {
			PlayerBehavior::MoveTo(target) => {
				entity.insert(SimpleMovement::<PlayerBehavior>::new(target))
			}
			PlayerBehavior::ShootGun(ray, cast, Side::Right) => {
				entity.insert(Skill::<PlayerBehavior>::new(
					ray,
					cast,
					Marker::<(Shoot, HandGun, Right)>::commands(),
				))
			}
			PlayerBehavior::ShootGun(ray, cast, Side::Left) => {
				entity.insert(Skill::<PlayerBehavior>::new(
					ray,
					cast,
					Marker::<(Shoot, HandGun, Left)>::commands(),
				))
			}
		};
	}
}

#[cfg(test)]
mod tests {
	use std::time::Duration;

	use super::*;
	use crate::components::{Cast, SimpleMovement, Skill};
	use bevy::prelude::{App, Commands, Entity, Ray, Update, Vec3};

	fn insert(entity: Entity, behavior: PlayerBehavior) -> impl FnMut(Commands) {
		move |mut commands| behavior.insert_into_entity(&mut commands.entity(entity))
	}

	#[test]
	fn insert_move_to() {
		let mut app = App::new();
		let behavior = PlayerBehavior::MoveTo(Vec3::ONE);
		let entity = app.world.spawn(()).id();

		app.add_systems(Update, insert(entity, behavior));
		app.update();

		let movement = app
			.world
			.entity(entity)
			.get::<SimpleMovement<PlayerBehavior>>();

		assert_eq!(
			Some(&SimpleMovement::<PlayerBehavior>::new(Vec3::ONE)),
			movement
		);
	}

	#[test]
	fn insert_shoot_right() {
		let ray = Ray {
			origin: Vec3::new(1., 2., 3.),
			direction: Vec3::new(3., 2., 1.),
		};
		let cast = Cast {
			pre: Duration::from_millis(1200),
			after: Duration::from_millis(3400),
		};
		let mut app = App::new();
		let behavior = PlayerBehavior::ShootGun(ray, cast, Side::Right);
		let entity = app.world.spawn(()).id();

		app.add_systems(Update, insert(entity, behavior));
		app.update();

		let movement = app.world.entity(entity).get::<Skill<PlayerBehavior>>();

		assert_eq!(
			Some(&Skill::<PlayerBehavior>::new(
				ray,
				cast,
				Marker::<(Shoot, HandGun, Right)>::commands()
			)),
			movement
		);
	}

	#[test]
	fn insert_shoot_left() {
		let ray = Ray {
			origin: Vec3::new(1., 2., 3.),
			direction: Vec3::new(3., 2., 1.),
		};
		let cast = Cast {
			pre: Duration::from_millis(1200),
			after: Duration::from_millis(3400),
		};
		let mut app = App::new();
		let behavior = PlayerBehavior::ShootGun(ray, cast, Side::Left);
		let entity = app.world.spawn(()).id();

		app.add_systems(Update, insert(entity, behavior));
		app.update();

		let movement = app.world.entity(entity).get::<Skill<PlayerBehavior>>();

		assert_eq!(
			Some(&Skill::<PlayerBehavior>::new(
				ray,
				cast,
				Marker::<(Shoot, HandGun, Left)>::commands()
			)),
			movement
		);
	}
}
