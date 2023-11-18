use super::Behavior;
use crate::components::{
	marker::{HandGun, Marker, Right, Shoot},
	Cast,
	Skill,
};
use bevy::{ecs::system::EntityCommands, math::Ray};
use std::time::Duration;

fn insert_fn(entity: &mut EntityCommands, ray: Ray) {
	entity.insert(Skill {
		ray,
		cast: Cast {
			pre: Duration::from_millis(300),
			after: Duration::from_millis(100),
		},
		marker_commands: Marker::<(Shoot, HandGun, Right)>::commands(),
		spawn_behavior: None,
	});
}

pub fn shoot_gun() -> Behavior {
	Behavior { insert_fn }
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::Skill;
	use bevy::prelude::{App, Commands, Entity, Ray, Update, Vec3};

	fn apply_behavior(behavior: Behavior, entity: Entity, ray: Ray) -> impl FnMut(Commands) {
		move |mut commands| {
			let mut entity = commands.entity(entity);
			behavior.insert_into(&mut entity, ray);
		}
	}

	#[test]
	fn add_skill() {
		let mut app = App::new();
		let behavior = shoot_gun();
		let entity = app.world.spawn(()).id();
		let ray = Ray {
			origin: Vec3::Y,
			direction: Vec3::NEG_Y,
		};

		app.add_systems(Update, apply_behavior(behavior, entity, ray));
		app.update();

		let skill = app.world.entity(entity).get::<Skill>();

		assert_eq!(
			Some(&Skill {
				ray,
				cast: Cast {
					pre: Duration::from_millis(300),
					after: Duration::from_millis(100)
				},
				marker_commands: Marker::<(Shoot, HandGun, Right)>::commands(),
				spawn_behavior: None,
			}),
			skill
		);
	}
}
