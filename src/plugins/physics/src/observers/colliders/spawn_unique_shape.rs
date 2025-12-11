use crate::components::colliders::{ColliderDefinition, ColliderOf, ColliderShape, Colliders};
use bevy::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl ColliderShape {
	pub(crate) fn spawn_unique(
		trigger: Trigger<OnInsert, ColliderDefinition>,
		mut commands: ZyheedaCommands,
		definitions: Query<(&ColliderDefinition, &Colliders)>,
	) {
		let entity = trigger.target();
		let Ok((definition, colliders)) = definitions.get(entity) else {
			return;
		};

		despawn_current_collider_shapes(&mut commands, colliders);
		spawn_collider_shape(commands, entity, definition);
	}
}

fn despawn_current_collider_shapes(commands: &mut ZyheedaCommands<'_, '_>, colliders: &Colliders) {
	for entity in colliders.iter() {
		commands.try_apply_on(&entity, |e| {
			e.try_despawn();
		});
	}
}

fn spawn_collider_shape(
	mut commands: ZyheedaCommands<'_, '_>,
	entity: Entity,
	ColliderDefinition(definition): &ColliderDefinition,
) {
	commands.spawn((
		ColliderOf(entity),
		ChildOf(entity),
		Transform::from_translation(definition.center_offset).with_rotation(definition.rotation),
		ColliderShape(definition.shape),
	));
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::handles_physics::colliders::{Collider, Shape};
	use std::f32::consts::PI;
	use testing::{SingleThreadedApp, assert_count, get_children};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(ColliderShape::spawn_unique);

		app
	}

	#[test]
	fn spawn_as_child() {
		let mut app = setup();
		let shape = Shape::Sphere { radius: 42. };

		let entity = app
			.world_mut()
			.spawn(ColliderDefinition(Collider::from_shape(shape)))
			.id();

		let [child] = assert_count!(1, get_children!(app, entity));
		assert_eq!(Some(&ColliderShape(shape)), child.get::<ColliderShape>());
	}

	#[test]
	fn spawn_with_offset() {
		let mut app = setup();
		let shape = Shape::Sphere { radius: 42. };
		let offset = Vec3::ONE;

		let entity = app
			.world_mut()
			.spawn(ColliderDefinition(
				Collider::from_shape(shape).with_center_offset(offset),
			))
			.id();

		let [child] = assert_count!(1, get_children!(app, entity));
		assert_eq!(
			Some(&Transform::from_translation(offset)),
			child.get::<Transform>(),
		);
	}

	#[test]
	fn spawn_with_rotation() {
		let mut app = setup();
		let shape = Shape::Sphere { radius: 42. };
		let rotation = Quat::from_rotation_x(PI);

		let entity = app
			.world_mut()
			.spawn(ColliderDefinition(
				Collider::from_shape(shape).with_rotation(rotation),
			))
			.id();

		let [child] = assert_count!(1, get_children!(app, entity));
		assert_eq!(
			Some(&Transform::from_rotation(rotation)),
			child.get::<Transform>(),
		);
	}

	#[test]
	fn reinsert_collider() {
		let mut app = setup();
		let shape = Shape::Sphere { radius: 42. };

		let entity = app
			.world_mut()
			.spawn(ColliderDefinition(Collider::from_shape(Shape::Sphere {
				radius: 11.,
			})))
			.insert(ColliderDefinition(Collider::from(shape)))
			.id();

		let [child] = assert_count!(1, get_children!(app, entity));
		assert_eq!(Some(&ColliderShape(shape)), child.get::<ColliderShape>());
	}
}
