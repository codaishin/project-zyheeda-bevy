use crate::components::OwnedBy;
use bevy::{
	ecs::{
		component::Component,
		entity::Entity,
		query::With,
		system::{Commands, Query},
	},
	hierarchy::DespawnRecursiveExt,
};

pub(crate) fn remove_not_owned<TOwner: Component>(
	mut commands: Commands,
	owners: Query<Entity, With<TOwner>>,
	owned: Query<(Entity, &OwnedBy<TOwner>)>,
) {
	let not_owned = |(.., owned): &(Entity, &OwnedBy<TOwner>)| owners.get(owned.owner).is_err();

	for (id, ..) in owned.iter().filter(not_owned) {
		remove(&mut commands, id);
	}
}

fn remove(commands: &mut Commands, id: Entity) {
	let Some(entity) = commands.get_entity(id) else {
		return;
	};
	entity.despawn_recursive();
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::test_tools::utils::SingleThreadedApp;
	use bevy::{
		app::{App, Update},
		hierarchy::BuildWorldChildren,
	};

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
		let owner_without_owner_component = app.world.spawn_empty().id();
		let owned = app
			.world
			.spawn(OwnedBy::<_Owner>::with(owner_without_owner_component))
			.id();

		app.update();

		assert!(app.world.get_entity(owned).is_none());
	}

	#[test]
	fn remove_recursive_when_owner_not_found() {
		let mut app = setup();
		let owner_without_owner_component = app.world.spawn_empty().id();
		let owned = app
			.world
			.spawn(OwnedBy::<_Owner>::with(owner_without_owner_component))
			.id();
		let child = app.world.spawn_empty().set_parent(owned).id();

		app.update();

		assert!(app.world.get_entity(child).is_none());
	}

	#[test]
	fn do_not_remove_when_owner_found() {
		let mut app = setup();
		let owner = app.world.spawn(_Owner).id();
		let owned = app.world.spawn(OwnedBy::<_Owner>::with(owner)).id();

		app.update();

		assert!(app.world.get_entity(owned).is_some());
	}
}
