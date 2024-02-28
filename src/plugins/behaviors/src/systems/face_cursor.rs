use crate::components::Face;
use bevy::{
	ecs::{
		entity::Entity,
		system::{Query, Res, Resource},
	},
	math::Vec3,
	transform::components::Transform,
};
use common::{components::ColliderRoot, resources::MouseHover, traits::intersect_at::IntersectAt};

pub(crate) fn face_cursor<TCursor: IntersectAt + Resource>(
	mut transforms: Query<&mut Transform>,
	agents: Query<(Entity, &Face)>,
	roots: Query<&ColliderRoot>,
	cursor: Res<TCursor>,
	hover: Res<MouseHover>,
) {
	let cursor_target = get_target(&transforms, &cursor, &hover);
	let face_targets = get_face_targets(&transforms, agents, roots, cursor_target);

	for (id, target) in face_targets {
		apply_facing(&mut transforms, id, target);
	}
}

fn apply_facing(transforms: &mut Query<&mut Transform>, id: Entity, target: Vec3) {
	let Ok(mut transform) = transforms.get_mut(id) else {
		return;
	};
	transform.look_at(target, Vec3::Y);
}

fn get_face_targets(
	transforms: &Query<&mut Transform>,
	agents: Query<(Entity, &Face)>,
	roots: Query<&ColliderRoot>,
	cursor_target: Option<Vec3>,
) -> Vec<(Entity, Vec3)> {
	agents
		.iter()
		.filter_map(|(id, face)| {
			let target = match face {
				Face::Cursor => cursor_target,
				Face::Entity(entity) => get_translation(get_root(*entity, &roots), transforms),
			};
			Some((id, target?))
		})
		.collect()
}

fn get_target<TCursor: IntersectAt + Resource>(
	transforms: &Query<&mut Transform>,
	cursor: &Res<TCursor>,
	hover: &Res<MouseHover>,
) -> Option<Vec3> {
	let Some(collider_info) = &hover.0 else {
		return cursor.intersect_at(0.);
	};

	let entity = collider_info.root.unwrap_or(collider_info.collider);
	get_translation(entity, transforms)
}

fn get_translation(entity: Entity, transforms: &Query<&mut Transform>) -> Option<Vec3> {
	transforms.get(entity).ok().map(|t| t.translation)
}

fn get_root(entity: Entity, roots: &Query<&ColliderRoot>) -> Entity {
	roots.get(entity).map(|r| r.0).unwrap_or(entity)
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		math::Vec3,
	};
	use common::{
		components::ColliderRoot,
		resources::ColliderInfo,
		test_tools::utils::SingleThreadedApp,
	};
	use mockall::{automock, predicate::eq};

	#[derive(Resource, Default)]
	struct _Cursor {
		mock: Mock_Cursor,
	}

	#[automock]
	impl IntersectAt for _Cursor {
		fn intersect_at(&self, height: f32) -> Option<Vec3> {
			self.mock.intersect_at(height)
		}
	}

	fn setup(cursor: _Cursor) -> App {
		let mut app = App::new_single_threaded([Update]);
		app.add_systems(Update, face_cursor::<_Cursor>);
		app.insert_resource(cursor);
		app.init_resource::<MouseHover>();

		app
	}

	#[test]
	fn do_face_cursor() {
		let mut cursor = _Cursor::default();
		cursor
			.mock
			.expect_intersect_at()
			.return_const(Vec3::new(1., 2., 3.));
		let mut app = setup(cursor);
		let agent = app
			.world
			.spawn((Transform::from_xyz(4., 5., 6.), Face::Cursor))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Transform::from_xyz(4., 5., 6.).looking_at(Vec3::new(1., 2., 3.), Vec3::Y)),
			agent.get::<Transform>()
		);
	}

	#[test]
	fn do_not_face_cursor_if_face_cursor_component_missing() {
		let mut cursor = _Cursor::default();
		cursor
			.mock
			.expect_intersect_at()
			.return_const(Vec3::new(1., 2., 3.));
		let mut app = setup(cursor);
		let agent = app.world.spawn(Transform::from_xyz(4., 5., 6.)).id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Transform::from_xyz(4., 5., 6.)),
			agent.get::<Transform>()
		);
	}

	#[test]
	fn use_zero_elevation_intersection() {
		let mut cursor = _Cursor::default();
		cursor
			.mock
			.expect_intersect_at()
			.with(eq(0.))
			.times(1)
			.return_const(None);
		let mut app = setup(cursor);
		app.world.spawn(Transform::from_xyz(4., 5., 6.));

		app.update();
	}

	#[test]
	fn face_hovering_collider_root() {
		let mut cursor = _Cursor::default();
		cursor
			.mock
			.expect_intersect_at()
			.return_const(Vec3::new(1., 2., 3.));
		let mut app = setup(cursor);

		let agent = app
			.world
			.spawn((Transform::from_xyz(4., 5., 6.), Face::Cursor))
			.id();

		let root = app.world.spawn(Transform::from_xyz(10., 11., 12.)).id();
		let collider = app.world.spawn(ColliderRoot(root)).id();
		app.insert_resource(MouseHover(Some(ColliderInfo {
			collider,
			root: Some(root),
		})));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Transform::from_xyz(4., 5., 6.).looking_at(Vec3::new(10., 11., 12.), Vec3::Y)),
			agent.get::<Transform>()
		);
	}

	#[test]
	fn face_hovering_collider() {
		let mut cursor = _Cursor::default();
		cursor
			.mock
			.expect_intersect_at()
			.return_const(Vec3::new(1., 2., 3.));
		let mut app = setup(cursor);

		let agent = app
			.world
			.spawn((Transform::from_xyz(4., 5., 6.), Face::Cursor))
			.id();

		let collider = app.world.spawn(Transform::from_xyz(10., 11., 12.)).id();
		app.insert_resource(MouseHover(Some(ColliderInfo {
			collider,
			root: None,
		})));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Transform::from_xyz(4., 5., 6.).looking_at(Vec3::new(10., 11., 12.), Vec3::Y)),
			agent.get::<Transform>()
		);
	}

	#[test]
	fn face_hovering_entity_with_collider_root() {
		let mut cursor = _Cursor::default();
		cursor
			.mock
			.expect_intersect_at()
			.return_const(Vec3::new(1., 2., 3.));
		let mut app = setup(cursor);

		let root = app.world.spawn(Transform::from_xyz(10., 11., 12.)).id();
		let collider = app.world.spawn(ColliderRoot(root)).id();
		let agent = app
			.world
			.spawn((Transform::from_xyz(4., 5., 6.), Face::Entity(collider)))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Transform::from_xyz(4., 5., 6.).looking_at(Vec3::new(10., 11., 12.), Vec3::Y)),
			agent.get::<Transform>()
		);
	}

	#[test]
	fn face_hovering_entity_with_no_collider_root() {
		let mut cursor = _Cursor::default();
		cursor
			.mock
			.expect_intersect_at()
			.return_const(Vec3::new(1., 2., 3.));
		let mut app = setup(cursor);

		let collider = app.world.spawn(Transform::from_xyz(10., 11., 12.)).id();
		let agent = app
			.world
			.spawn((Transform::from_xyz(4., 5., 6.), Face::Entity(collider)))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Transform::from_xyz(4., 5., 6.).looking_at(Vec3::new(10., 11., 12.), Vec3::Y)),
			agent.get::<Transform>()
		);
	}
}
