use super::InsertIntoEntity;
use crate::{
	behaviors::Behavior,
	components::{
		marker::{HandGun, Left, Marker, Right, Shoot},
		Side,
		SimpleMovement,
		Skill,
	},
};
use bevy::ecs::system::EntityCommands;

impl InsertIntoEntity for Behavior {
	fn insert_into_entity(self, entity: &mut EntityCommands) {
		match self {
			Behavior::MoveTo(target) => entity.insert(SimpleMovement::<Behavior>::new(target)),
			Behavior::ShootGun(ray, cast, Side::Right) => entity.insert(Skill::<Behavior>::new(
				ray,
				cast,
				Marker::<(Shoot, HandGun, Right)>::commands(),
			)),
			Behavior::ShootGun(ray, cast, Side::Left) => entity.insert(Skill::<Behavior>::new(
				ray,
				cast,
				Marker::<(Shoot, HandGun, Left)>::commands(),
			)),
		};
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{Cast, Seconds, SimpleMovement, Skill};
	use bevy::prelude::{App, Commands, Entity, Ray, Update, Vec3};

	fn insert(entity: Entity, behavior: Behavior) -> impl FnMut(Commands) {
		move |mut commands| behavior.insert_into_entity(&mut commands.entity(entity))
	}

	#[test]
	fn insert_move_to() {
		let mut app = App::new();
		let behavior = Behavior::MoveTo(Vec3::ONE);
		let entity = app.world.spawn(()).id();

		app.add_systems(Update, insert(entity, behavior));
		app.update();

		let movement = app.world.entity(entity).get::<SimpleMovement<Behavior>>();

		assert_eq!(Some(&SimpleMovement::<Behavior>::new(Vec3::ONE)), movement);
	}

	#[test]
	fn insert_shoot_right() {
		let ray = Ray {
			origin: Vec3::new(1., 2., 3.),
			direction: Vec3::new(3., 2., 1.),
		};
		let cast = Cast {
			pre: Seconds(1.2),
			after: Seconds(3.4),
		};
		let mut app = App::new();
		let behavior = Behavior::ShootGun(ray, cast, Side::Right);
		let entity = app.world.spawn(()).id();

		app.add_systems(Update, insert(entity, behavior));
		app.update();

		let movement = app.world.entity(entity).get::<Skill<Behavior>>();

		assert_eq!(
			Some(&Skill::<Behavior>::new(
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
			pre: Seconds(1.2),
			after: Seconds(3.4),
		};
		let mut app = App::new();
		let behavior = Behavior::ShootGun(ray, cast, Side::Left);
		let entity = app.world.spawn(()).id();

		app.add_systems(Update, insert(entity, behavior));
		app.update();

		let movement = app.world.entity(entity).get::<Skill<Behavior>>();

		assert_eq!(
			Some(&Skill::<Behavior>::new(
				ray,
				cast,
				Marker::<(Shoot, HandGun, Left)>::commands()
			)),
			movement
		);
	}
}
