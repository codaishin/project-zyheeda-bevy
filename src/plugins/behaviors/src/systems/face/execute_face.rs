use std::ops::Deref;

use bevy::prelude::*;
use common::{
	components::{
		collider_relationship::ColliderOfInteractionTarget,
		immobilized::Immobilized,
		persistent_entity::PersistentEntity,
	},
	tools::collider_info::ColliderInfo,
	traits::{
		accessors::get::{GetMut, Getter, RefInto},
		handles_orientation::Face,
		intersect_at::IntersectAt,
	},
	zyheeda_commands::ZyheedaCommands,
};

pub(crate) fn execute_face<TMouseHover, TCursor>(
	In(faces): In<Vec<(Entity, Face)>>,
	mut transforms: Query<(Entity, &mut Transform, Option<&Immobilized>)>,
	commands: ZyheedaCommands,
	colliders: Query<&ColliderOfInteractionTarget>,
	cursor: Res<TCursor>,
	hover: Res<TMouseHover>,
) where
	TMouseHover: Resource + for<'a> RefInto<'a, Option<&'a ColliderInfo<Entity>>>,
	TCursor: IntersectAt + Resource,
{
	let Some(target) = get_cursor_target(&transforms, cursor.deref(), hover.deref()) else {
		return;
	};

	for (id, target) in get_face_targets(&transforms, faces, colliders, target, commands) {
		apply_facing(&mut transforms, id, target);
	}
}

fn apply_facing(
	transforms: &mut Query<(Entity, &mut Transform, Option<&Immobilized>)>,
	id: Entity,
	target: Vec3,
) {
	let Ok((_, mut transform, immobilized)) = transforms.get_mut(id) else {
		return;
	};
	if immobilized.is_some() {
		return;
	}
	transform.look_at(target, Vec3::Y);
}

fn get_face_targets(
	transforms: &Query<(Entity, &mut Transform, Option<&Immobilized>)>,
	faces: Vec<(Entity, Face)>,
	colliders: Query<&ColliderOfInteractionTarget>,
	(target_entity, target): (Option<Entity>, Vec3),
	mut commands: ZyheedaCommands,
) -> Vec<(Entity, Vec3)> {
	faces
		.iter()
		.filter(|(id, _)| Some(*id) != target_entity)
		.filter_map(|(id, face)| {
			let target = match *face {
				Face::Translation(translation) => Some(translation),
				Face::Cursor => Some(target),
				Face::Entity(entity) => {
					let target = get_target(entity, &colliders, &mut commands)?;
					get_translation(target, transforms)
				}
			};
			Some((*id, target?))
		})
		.collect()
}

fn get_cursor_target<TMouseHover, TCursor>(
	transforms: &Query<(Entity, &mut Transform, Option<&Immobilized>)>,
	cursor: &TCursor,
	hover: &TMouseHover,
) -> Option<(Option<Entity>, Vec3)>
where
	TMouseHover: Resource + for<'a> RefInto<'a, Option<&'a ColliderInfo<Entity>>>,
	TCursor: IntersectAt + Resource,
{
	let Some(collider_info) = hover.get::<Option<&ColliderInfo<Entity>>>() else {
		return cursor.intersect_at(0.).map(|t| (None, t));
	};

	let entity = collider_info.root.unwrap_or(collider_info.collider);

	get_translation(entity, transforms).map(|t| (Some(entity), t))
}

fn get_translation(
	entity: Entity,
	transforms: &Query<(Entity, &mut Transform, Option<&Immobilized>)>,
) -> Option<Vec3> {
	transforms.get(entity).ok().map(|(_, t, _)| t.translation)
}

fn get_target(
	entity: PersistentEntity,
	roots: &Query<&ColliderOfInteractionTarget>,
	commands: &mut ZyheedaCommands,
) -> Option<Entity> {
	let entity = commands.get_mut(&entity)?.id();

	Some(
		roots
			.get(entity)
			.map(ColliderOfInteractionTarget::target)
			.unwrap_or(entity),
	)
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::{component::Component, system::IntoSystem},
		math::Vec3,
	};
	use common::{
		components::persistent_entity::PersistentEntity,
		traits::register_persistent_entities::RegisterPersistentEntities,
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Resource, NestedMocks)]
	struct _Cursor {
		mock: Mock_Cursor,
	}

	#[automock]
	impl IntersectAt for _Cursor {
		fn intersect_at(&self, height: f32) -> Option<Vec3> {
			self.mock.intersect_at(height)
		}
	}

	#[derive(Resource, Default)]
	struct _MouseHover(Option<ColliderInfo<Entity>>);

	impl<'a> RefInto<'a, Option<&'a ColliderInfo<Entity>>> for _MouseHover {
		fn ref_into(&self) -> Option<&ColliderInfo<Entity>> {
			self.0.as_ref()
		}
	}

	#[derive(Component)]
	struct _Face(Face);

	fn read_faces(query: Query<(Entity, &_Face)>) -> Vec<(Entity, Face)> {
		query.iter().map(|(id, face)| (id, face.0)).collect()
	}

	fn setup(cursor: _Cursor) -> App {
		let mut app = App::new().single_threaded(Update);

		app.register_persistent_entities();
		app.add_systems(
			Update,
			read_faces.pipe(execute_face::<_MouseHover, _Cursor>),
		);
		app.insert_resource(cursor);
		app.init_resource::<_MouseHover>();

		app
	}

	#[test]
	fn do_face_cursor() {
		let mut app = setup(_Cursor::new().with_mock(|mock| {
			mock.expect_intersect_at()
				.return_const(Vec3::new(1., 2., 3.));
		}));
		let agent = app
			.world_mut()
			.spawn((Transform::from_xyz(4., 5., 6.), _Face(Face::Cursor)))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&Transform::from_xyz(4., 5., 6.).looking_at(Vec3::new(1., 2., 3.), Vec3::Y)),
			agent.get::<Transform>()
		);
	}

	#[test]
	fn do_not_face_cursor_if_face_cursor_component_missing() {
		let mut app = setup(_Cursor::new().with_mock(|mock| {
			mock.expect_intersect_at()
				.return_const(Vec3::new(1., 2., 3.));
		}));
		let agent = app.world_mut().spawn(Transform::from_xyz(4., 5., 6.)).id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&Transform::from_xyz(4., 5., 6.)),
			agent.get::<Transform>()
		);
	}

	#[test]
	fn use_zero_elevation_intersection() {
		let mut app = setup(_Cursor::new().with_mock(|mock| {
			mock.expect_intersect_at()
				.with(eq(0.))
				.times(1)
				.return_const(None);
		}));
		app.world_mut().spawn(Transform::from_xyz(4., 5., 6.));

		app.update();
	}

	#[test]
	fn face_hovering_collider_root() {
		let mut app = setup(_Cursor::new().with_mock(|mock| {
			mock.expect_intersect_at()
				.return_const(Vec3::new(1., 2., 3.));
		}));
		let agent = app
			.world_mut()
			.spawn((Transform::from_xyz(4., 5., 6.), _Face(Face::Cursor)))
			.id();
		let root = app
			.world_mut()
			.spawn(Transform::from_xyz(10., 11., 12.))
			.id();
		let collider = app
			.world_mut()
			.spawn(ColliderOfInteractionTarget::from_raw(root))
			.id();
		app.insert_resource(_MouseHover(Some(ColliderInfo {
			collider,
			root: Some(root),
		})));

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&Transform::from_xyz(4., 5., 6.).looking_at(Vec3::new(10., 11., 12.), Vec3::Y)),
			agent.get::<Transform>()
		);
	}

	#[test]
	fn do_not_default_rotation_when_looking_at_self_via_root() {
		let mut app = setup(_Cursor::new().with_mock(|mock| {
			mock.expect_intersect_at()
				.return_const(Vec3::new(1., 2., 3.));
		}));
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(4., 5., 6.).looking_to(Vec3::new(1., 0., 1.), Vec3::Y),
				_Face(Face::Cursor),
			))
			.id();
		let collider = app
			.world_mut()
			.spawn(ColliderOfInteractionTarget::from_raw(agent))
			.id();
		app.insert_resource(_MouseHover(Some(ColliderInfo {
			collider,
			root: Some(agent),
		})));

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&Transform::from_xyz(4., 5., 6.).looking_to(Vec3::new(1., 0., 1.), Vec3::Y)),
			agent.get::<Transform>()
		);
	}

	#[test]
	fn face_hovering_collider() {
		let mut app = setup(_Cursor::new().with_mock(|mock| {
			mock.expect_intersect_at()
				.return_const(Vec3::new(1., 2., 3.));
		}));
		let agent = app
			.world_mut()
			.spawn((Transform::from_xyz(4., 5., 6.), _Face(Face::Cursor)))
			.id();
		let collider = app
			.world_mut()
			.spawn(Transform::from_xyz(10., 11., 12.))
			.id();
		app.insert_resource(_MouseHover(Some(ColliderInfo {
			collider,
			root: None,
		})));

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&Transform::from_xyz(4., 5., 6.).looking_at(Vec3::new(10., 11., 12.), Vec3::Y)),
			agent.get::<Transform>()
		);
	}

	#[test]
	fn do_not_default_rotation_when_looking_at_self_via_collider() {
		let mut app = setup(_Cursor::new().with_mock(|mock| {
			mock.expect_intersect_at()
				.return_const(Vec3::new(1., 2., 3.));
		}));
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(4., 5., 6.).looking_to(Vec3::new(1., 0., 1.), Vec3::Y),
				_Face(Face::Cursor),
			))
			.id();
		app.insert_resource(_MouseHover(Some(ColliderInfo {
			collider: agent,
			root: None,
		})));

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&Transform::from_xyz(4., 5., 6.).looking_to(Vec3::new(1., 0., 1.), Vec3::Y)),
			agent.get::<Transform>()
		);
	}

	#[test]
	fn face_hovering_entity_with_collider_root() {
		let mut app = setup(_Cursor::new().with_mock(|mock| {
			mock.expect_intersect_at()
				.return_const(Vec3::new(1., 2., 3.));
		}));
		let root = app
			.world_mut()
			.spawn(Transform::from_xyz(10., 11., 12.))
			.id();
		let collider = PersistentEntity::default();
		app.world_mut()
			.spawn((ColliderOfInteractionTarget::from_raw(root), collider));
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(4., 5., 6.),
				_Face(Face::Entity(collider)),
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&Transform::from_xyz(4., 5., 6.).looking_at(Vec3::new(10., 11., 12.), Vec3::Y)),
			agent.get::<Transform>()
		);
	}

	#[test]
	fn face_hovering_entity_with_no_collider_root() {
		let mut app = setup(_Cursor::new().with_mock(|mock| {
			mock.expect_intersect_at()
				.return_const(Vec3::new(1., 2., 3.));
		}));
		let collider = PersistentEntity::default();
		app.world_mut()
			.spawn((Transform::from_xyz(10., 11., 12.), collider));
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(4., 5., 6.),
				_Face(Face::Entity(collider)),
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&Transform::from_xyz(4., 5., 6.).looking_at(Vec3::new(10., 11., 12.), Vec3::Y)),
			agent.get::<Transform>()
		);
	}

	#[test]
	fn face_translation() {
		let mut app = setup(_Cursor::new().with_mock(|mock| {
			mock.expect_intersect_at()
				.return_const(Vec3::new(1., 2., 3.));
		}));
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(4., 5., 6.),
				_Face(Face::Translation(Vec3::new(10., 11., 12.))),
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&Transform::from_xyz(4., 5., 6.).looking_at(Vec3::new(10., 11., 12.), Vec3::Y)),
			agent.get::<Transform>()
		);
	}

	#[test]
	fn do_not_face_cursor_when_agent_immobilized() {
		let mut app = setup(_Cursor::new().with_mock(|mock| {
			mock.expect_intersect_at()
				.return_const(Vec3::new(1., 2., 3.));
		}));
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(4., 5., 6.),
				_Face(Face::Cursor),
				Immobilized,
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&Transform::from_xyz(4., 5., 6.)),
			agent.get::<Transform>()
		);
	}

	#[test]
	fn face_hovering_collider_when_collided_immobilized() {
		let mut app = setup(_Cursor::new().with_mock(|mock| {
			mock.expect_intersect_at()
				.return_const(Vec3::new(1., 2., 3.));
		}));
		let agent = app
			.world_mut()
			.spawn((Transform::from_xyz(4., 5., 6.), _Face(Face::Cursor)))
			.id();
		let collider = app
			.world_mut()
			.spawn((Transform::from_xyz(10., 11., 12.), Immobilized))
			.id();
		app.insert_resource(_MouseHover(Some(ColliderInfo {
			collider,
			root: None,
		})));

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&Transform::from_xyz(4., 5., 6.).looking_at(Vec3::new(10., 11., 12.), Vec3::Y)),
			agent.get::<Transform>()
		);
	}
}
