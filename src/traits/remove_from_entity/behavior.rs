use super::RemoveFromEntity;
use crate::{behaviors::Behavior, components::SimpleMovement};

impl RemoveFromEntity for Behavior {
	fn remove_from_entity(&self, entity: &mut bevy::ecs::system::EntityCommands) {
		match self {
			Behavior::MoveTo(_) => entity.remove::<SimpleMovement<Behavior>>(),
		};
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::SimpleMovement;
	use bevy::prelude::{App, Commands, Entity, Update, Vec3};

	fn remove(entity: Entity, behavior: Behavior) -> impl FnMut(Commands) {
		move |mut commands| behavior.remove_from_entity(&mut commands.entity(entity))
	}

	#[test]
	fn remove_move_to() {
		let mut app = App::new();
		let behavior = Behavior::MoveTo(Vec3::ONE);
		let entity = app
			.world
			.spawn(SimpleMovement::<Behavior>::new(Vec3::ONE))
			.id();

		app.add_systems(Update, remove(entity, behavior));
		app.update();

		let entity = app.world.entity(entity);

		assert!(!entity.contains::<SimpleMovement<Behavior>>());
	}
}
