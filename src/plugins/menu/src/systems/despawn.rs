use bevy::prelude::*;

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
	use super::*;

	#[derive(Component)]
	struct _Component;

	#[test]
	fn despawn_entity() {
		let mut app = App::new();

		let entity = app.world_mut().spawn(_Component).id();

		app.add_systems(Update, despawn::<_Component>);
		app.update();

		let entity = app.world().get_entity(entity).ok();

		assert!(entity.is_none());
	}

	#[test]
	fn despawn_entity_children() {
		let mut app = App::new();

		let entity = app.world_mut().spawn(_Component).id();
		let child = app.world_mut().spawn(()).id();
		app.world_mut().entity_mut(entity).add_children(&[child]);

		app.add_systems(Update, despawn::<_Component>);
		app.update();

		let child = app.world().get_entity(child).ok();

		assert!(child.is_none());
	}
}
