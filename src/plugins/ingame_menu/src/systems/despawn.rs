use bevy::{
	ecs::{
		component::Component,
		query::With,
		system::{Commands, Query},
	},
	hierarchy::DespawnRecursiveExt,
	prelude::Entity,
};

pub fn despawn<TComponent: Component>(
	mut commands: Commands,
	entities: Query<Entity, With<TComponent>>,
) {
	for entity in &entities {
		commands.entity(entity).despawn_recursive();
	}
}

#[cfg(test)]
mod tests {
	use bevy::{
		app::{App, Update},
		hierarchy::BuildWorldChildren,
	};

	use super::*;

	#[derive(Component)]
	struct _Component;

	#[test]
	fn despawn_entity() {
		let mut app = App::new();

		let entity = app.world.spawn(_Component).id();

		app.add_systems(Update, despawn::<_Component>);
		app.update();

		let entity = app.world.get_entity(entity);

		assert!(entity.is_none());
	}

	#[test]
	fn despawn_entity_children() {
		let mut app = App::new();

		let entity = app.world.spawn(_Component).id();
		let child = app.world.spawn(()).id();
		app.world.entity_mut(entity).push_children(&[child]);

		app.add_systems(Update, despawn::<_Component>);
		app.update();

		let child = app.world.get_entity(child);

		assert!(child.is_none());
	}
}
