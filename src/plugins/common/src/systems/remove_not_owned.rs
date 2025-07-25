use crate::components::ui_node_for::UiNodeFor;
use bevy::prelude::*;

pub(crate) fn remove_not_owned<TOwner: Component>(
	mut commands: Commands,
	owners: Query<Entity, With<TOwner>>,
	owned: Query<(Entity, &UiNodeFor<TOwner>)>,
) {
	let not_owned = |(.., owned): &(Entity, &UiNodeFor<TOwner>)| owners.get(owned.owner).is_err();

	for (id, ..) in owned.iter().filter(not_owned) {
		remove(&mut commands, id);
	}
}

fn remove(commands: &mut Commands, id: Entity) {
	let Ok(mut entity) = commands.get_entity(id) else {
		return;
	};
	entity.despawn();
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Owner;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, remove_not_owned::<_Owner>);

		app
	}

	#[test]
	fn remove_when_owner_not_found() {
		let mut app = setup();
		let owner_without_owner_component = app.world_mut().spawn_empty().id();
		let owned = app
			.world_mut()
			.spawn(UiNodeFor::<_Owner>::with(owner_without_owner_component))
			.id();

		app.update();

		assert!(app.world().get_entity(owned).is_err());
	}

	#[test]
	fn remove_recursive_when_owner_not_found() {
		let mut app = setup();
		let owner_without_owner_component = app.world_mut().spawn_empty().id();
		let owned = app
			.world_mut()
			.spawn(UiNodeFor::<_Owner>::with(owner_without_owner_component))
			.id();
		let child = app.world_mut().spawn_empty().insert(ChildOf(owned)).id();

		app.update();

		assert!(app.world().get_entity(child).is_err());
	}

	#[test]
	fn do_not_remove_when_owner_found() {
		let mut app = setup();
		let owner = app.world_mut().spawn(_Owner).id();
		let owned = app.world_mut().spawn(UiNodeFor::<_Owner>::with(owner)).id();

		app.update();

		assert!(app.world().get_entity(owned).is_ok());
	}
}
