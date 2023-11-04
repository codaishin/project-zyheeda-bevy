use super::InsertIntoEntity;
use crate::{behaviors::Behavior, components::SimpleMovement};
use bevy::ecs::system::EntityCommands;

impl InsertIntoEntity for Behavior {
	fn insert_into_entity(self, entity: &mut EntityCommands) {
		match self {
			Behavior::MoveTo(target) => entity.insert(SimpleMovement { target }),
		};
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::SimpleMovement;
	use bevy::prelude::{App, Commands, Entity, Update, Vec3};

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

		let movement = app.world.entity(entity).get::<SimpleMovement>();

		assert_eq!(Some(&SimpleMovement { target: Vec3::ONE }), movement);
	}
}
