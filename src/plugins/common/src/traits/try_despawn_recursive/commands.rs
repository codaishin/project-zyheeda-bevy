use super::TryDespawnRecursive;
use bevy::prelude::*;

impl<'w, 's> TryDespawnRecursive for Commands<'w, 's> {
	fn try_despawn_recursive(&mut self, entity: Entity) {
		let Some(entity) = self.get_entity(entity) else {
			return;
		};
		entity.despawn_recursive();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::RunSystemOnce;

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn despawn_entity() {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut commands: Commands| commands.try_despawn_recursive(entity));

		assert!(app.world().get_entity(entity).is_none());
	}

	#[test]
	fn despawn_entity_children() {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();
		let child = app.world_mut().spawn_empty().set_parent(entity).id();

		app.world_mut()
			.run_system_once(move |mut commands: Commands| commands.try_despawn_recursive(entity));

		assert!(app.world().get_entity(child).is_none());
	}

	#[test]
	fn no_panic_when_entity_does_not_exist() {
		let mut app = setup();
		let entity = Entity::from_raw(1000);

		app.world_mut()
			.run_system_once(move |mut commands: Commands| commands.try_despawn_recursive(entity));
	}
}