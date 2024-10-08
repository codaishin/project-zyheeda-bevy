use crate::components::Face;
use bevy::{
	ecs::{
		entity::Entity,
		query::QueryFilter,
		system::{In, Query, Res, Resource},
	},
	math::Vec3,
	transform::components::Transform,
};
use common::{
	components::{ColliderRoot, Immobilized},
	resources::MouseHover,
	traits::intersect_at::IntersectAt,
};

pub(crate) fn execute_face<TCursor: IntersectAt + Resource, TCursorFilter: QueryFilter>(
	faces: In<Vec<(Entity, Face)>>,
	mut transforms: Query<(&mut Transform, Option<&Immobilized>)>,
	roots: Query<&ColliderRoot>,
	cursor: Res<TCursor>,
	hover: Res<MouseHover>,
	cursor_filter: Query<(), TCursorFilter>,
) {
	let cursor_target = get_cursor_target(&transforms, &cursor, &hover, &cursor_filter);
	let face_targets = get_face_targets(&transforms, faces.0, roots, cursor_target);

	for (id, target) in face_targets {
		apply_facing(&mut transforms, id, target);
	}
}

fn apply_facing(
	transforms: &mut Query<(&mut Transform, Option<&Immobilized>)>,
	id: Entity,
	target: Vec3,
) {
	let Ok((mut transform, immobilized)) = transforms.get_mut(id) else {
		return;
	};
	if immobilized.is_some() {
		return;
	}
	transform.look_at(target, Vec3::Y);
}

fn get_face_targets(
	transforms: &Query<(&mut Transform, Option<&Immobilized>)>,
	faces: Vec<(Entity, Face)>,
	roots: Query<&ColliderRoot>,
	cursor_target: Option<Vec3>,
) -> Vec<(Entity, Vec3)> {
	faces
		.iter()
		.filter_map(|(id, face)| {
			let target = match *face {
				Face::Cursor => cursor_target,
				Face::Entity(entity) => get_translation(get_root(entity, &roots), transforms),
				Face::Translation(translation) => Some(translation),
			};
			Some((*id, target?))
		})
		.collect()
}

fn get_cursor_target<TCursor: IntersectAt + Resource, TFilter: QueryFilter>(
	transforms: &Query<(&mut Transform, Option<&Immobilized>)>,
	cursor: &Res<TCursor>,
	hover: &Res<MouseHover>,
	cursor_filter: &Query<(), TFilter>,
) -> Option<Vec3> {
	let Some(collider_info) = &hover.0 else {
		return cursor.intersect_at(0.);
	};

	let entity = collider_info.root.unwrap_or(collider_info.collider);

	match cursor_filter.get(entity) {
		Ok(_) => get_translation(entity, transforms),
		Err(_) => cursor.intersect_at(0.),
	}
}

fn get_translation(
	entity: Entity,
	transforms: &Query<(&mut Transform, Option<&Immobilized>)>,
) -> Option<Vec3> {
	transforms.get(entity).ok().map(|(t, _)| t.translation)
}

fn get_root(entity: Entity, roots: &Query<&ColliderRoot>) -> Entity {
	roots.get(entity).map(|r| r.0).unwrap_or(entity)
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::{component::Component, system::IntoSystem},
		math::Vec3,
		prelude::Without,
	};
	use common::{
		components::{ColliderRoot, Immobilized},
		resources::ColliderInfo,
		test_tools::utils::SingleThreadedApp,
		traits::nested_mock::NestedMocks,
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Resource, NestedMocks)]
	struct _Cursor {
		mock: Mock_Cursor,
	}

	#[derive(Component)]
	struct _Face(Face);

	#[derive(Component)]
	struct _Ignore;

	#[automock]
	impl IntersectAt for _Cursor {
		fn intersect_at(&self, height: f32) -> Option<Vec3> {
			self.mock.intersect_at(height)
		}
	}

	fn read_faces(query: Query<(Entity, &_Face)>) -> Vec<(Entity, Face)> {
		query.iter().map(|(id, face)| (id, face.0)).collect()
	}

	fn setup(cursor: _Cursor) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			read_faces.pipe(execute_face::<_Cursor, Without<_Ignore>>),
		);
		app.insert_resource(cursor);
		app.init_resource::<MouseHover>();

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
		let collider = app.world_mut().spawn(ColliderRoot(root)).id();
		app.insert_resource(MouseHover(Some(ColliderInfo {
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
	fn do_not_face_hovering_entity_with_collider_root_with_non_matching_filter() {
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
			.spawn((Transform::from_xyz(10., 11., 12.), _Ignore))
			.id();
		let collider = app.world_mut().spawn(ColliderRoot(root)).id();
		app.insert_resource(MouseHover(Some(ColliderInfo {
			collider,
			root: Some(root),
		})));

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&Transform::from_xyz(4., 5., 6.).looking_at(Vec3::new(1., 2., 3.), Vec3::Y)),
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
		app.insert_resource(MouseHover(Some(ColliderInfo {
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
	fn do_not_face_hovering_collider_with_non_matching_filter() {
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
			.spawn((Transform::from_xyz(10., 11., 12.), _Ignore))
			.id();
		app.insert_resource(MouseHover(Some(ColliderInfo {
			collider,
			root: None,
		})));

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&Transform::from_xyz(4., 5., 6.).looking_at(Vec3::new(1., 2., 3.), Vec3::Y)),
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
		let collider = app.world_mut().spawn(ColliderRoot(root)).id();
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
		let collider = app
			.world_mut()
			.spawn(Transform::from_xyz(10., 11., 12.))
			.id();
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
		app.insert_resource(MouseHover(Some(ColliderInfo {
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
