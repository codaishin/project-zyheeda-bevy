use crate::components::face_target::FaceTarget;
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	self,
	traits::{
		accessors::get::Get,
		handles_orientation::{Face, FaceTargetIs},
		handles_physics::{MouseHover, MouseHoversOver, Raycast},
	},
	zyheeda_commands::ZyheedaCommands,
};
use std::ops::DerefMut;

pub(crate) fn execute_face<TMouseHover>(
	In(faces): In<Vec<(Entity, Face)>>,
	mut transforms: Query<&mut Transform>,
	face_targets: Query<&FaceTarget>,
	commands: ZyheedaCommands,
	mut hover: StaticSystemParam<TMouseHover>,
) where
	TMouseHover: for<'w, 's> SystemParam<Item<'w, 's>: Raycast<MouseHover>>,
{
	for (entity, face) in faces {
		let target = match face {
			Face::Translation(translation) => Some(translation),
			Face::Target => get_target(entity, hover.deref_mut(), &transforms, &face_targets),
			Face::Entity(entity) => get_translation(commands.get(&entity), &transforms),
			Face::Direction(dir) => get_translation(Some(entity), &transforms).map(|tr| tr + *dir),
		};
		let Some(target) = target else {
			continue;
		};
		apply_facing(&mut transforms, entity, target);
	}
}

fn apply_facing(transforms: &mut Query<&mut Transform>, id: Entity, target: Vec3) {
	let Ok(mut transform) = transforms.get_mut(id) else {
		return;
	};
	let target = target.with_y(transform.translation.y);
	if transform.translation == target {
		return;
	}
	transform.look_at(target, Vec3::Y);
}

fn get_target<TMouseHover>(
	entity: Entity,
	hover: &mut TMouseHover,
	transforms: &Query<&mut Transform>,
	face_targets: &Query<&FaceTarget>,
) -> Option<Vec3>
where
	TMouseHover: Raycast<MouseHover>,
{
	if let Ok(FaceTargetIs::Entity(target)) = face_targets.get(entity).map(|t| t.0) {
		return transforms.get(target).ok().map(|tr| tr.translation);
	}

	let hover = hover.raycast(MouseHover {
		exclude: vec![entity],
	})?;

	match hover {
		MouseHoversOver::Ground { point } => Some(point),
		MouseHoversOver::Object { entity, .. } => {
			transforms.get(entity).ok().map(|tr| tr.translation)
		}
	}
}

fn get_translation(entity: Option<Entity>, transforms: &Query<&mut Transform>) -> Option<Vec3> {
	transforms.get(entity?).ok().map(|t| t.translation)
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
		traits::{
			handles_orientation::FaceTargetIs,
			register_persistent_entities::RegisterPersistentEntities,
		},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Resource, NestedMocks)]
	struct _RayCast {
		mock: Mock_RayCast,
	}

	#[automock]
	impl Raycast<MouseHover> for _RayCast {
		fn raycast(&mut self, args: MouseHover) -> Option<MouseHoversOver> {
			self.mock.raycast(args)
		}
	}

	#[derive(Component)]
	struct _Face(Face);

	fn read_faces(query: Query<(Entity, &_Face)>) -> Vec<(Entity, Face)> {
		query.iter().map(|(id, face)| (id, face.0)).collect()
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.register_persistent_entities();
		app.add_systems(Update, read_faces.pipe(execute_face::<ResMut<_RayCast>>));
		app.insert_resource(_RayCast::new().with_mock(|mock| {
			mock.expect_raycast().never();
		}));

		app
	}

	#[test]
	fn do_face_cursor_ground() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((Transform::from_xyz(4., 2., 6.), _Face(Face::Target)))
			.id();
		app.insert_resource(_RayCast::new().with_mock(|mock| {
			mock.expect_raycast()
				.with(eq(MouseHover {
					exclude: vec![agent],
				}))
				.return_const(MouseHoversOver::Ground {
					point: Vec3::new(1., 2., 3.),
				});
		}));

		app.update();

		assert_eq!(
			Some(&Transform::from_xyz(4., 2., 6.).looking_at(Vec3::new(1., 2., 3.), Vec3::Y)),
			app.world().entity(agent).get::<Transform>()
		);
	}

	#[test]
	fn face_target_hover_entity() {
		let mut app = setup();
		let entity = app.world_mut().spawn(Transform::from_xyz(6., 5., 20.)).id();
		let agent = app
			.world_mut()
			.spawn((Transform::from_xyz(4., 5., 6.), _Face(Face::Target)))
			.id();
		app.insert_resource(_RayCast::new().with_mock(|mock| {
			mock.expect_raycast()
				.with(eq(MouseHover {
					exclude: vec![agent],
				}))
				.return_const(MouseHoversOver::Object {
					entity,
					point: Vec3::new(4., 5., 7.),
				});
		}));

		app.update();

		assert_eq!(
			Some(&Transform::from_xyz(4., 5., 6.).looking_at(Vec3::new(6., 5., 20.), Vec3::Y)),
			app.world().entity(agent).get::<Transform>()
		);
	}

	#[test]
	fn face_target_hover_ground() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((Transform::from_xyz(4., 5., 6.), _Face(Face::Target)))
			.id();
		app.insert_resource(_RayCast::new().with_mock(|mock| {
			mock.expect_raycast()
				.with(eq(MouseHover {
					exclude: vec![agent],
				}))
				.return_const(MouseHoversOver::Ground {
					point: Vec3::new(6., 3., 7.),
				});
		}));

		app.update();

		assert_eq!(
			Some(&Transform::from_xyz(4., 5., 6.).looking_at(Vec3::new(6., 5., 7.), Vec3::Y)),
			app.world().entity(agent).get::<Transform>()
		);
	}

	#[test]
	fn face_target_entity_from_target_definition() {
		let mut app = setup();
		let entity = app.world_mut().spawn(Transform::from_xyz(6., 5., 20.)).id();
		let target_definition = app.world_mut().spawn(Transform::from_xyz(11., 1., 2.)).id();
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(4., 5., 6.),
				_Face(Face::Target),
				FaceTarget(FaceTargetIs::Entity(target_definition)),
			))
			.id();
		app.insert_resource(_RayCast::new().with_mock(|mock| {
			mock.expect_raycast()
				.with(eq(MouseHover {
					exclude: vec![agent],
				}))
				.return_const(MouseHoversOver::Object {
					entity,
					point: Vec3::new(4., 5., 7.),
				});
		}));

		app.update();

		assert_eq!(
			Some(&Transform::from_xyz(4., 5., 6.).looking_at(Vec3::new(11., 5., 2.), Vec3::Y)),
			app.world().entity(agent).get::<Transform>()
		);
	}

	#[test]
	fn face_hovering_entity() {
		let mut app = setup();
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

		assert_eq!(
			Some(&Transform::from_xyz(4., 5., 6.).looking_at(Vec3::new(10., 5., 12.), Vec3::Y)),
			app.world().entity(agent).get::<Transform>()
		);
	}

	#[test]
	fn face_translation() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(4., 5., 6.),
				_Face(Face::Translation(Vec3::new(10., 5., 12.))),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&Transform::from_xyz(4., 5., 6.).looking_at(Vec3::new(10., 5., 12.), Vec3::Y)),
			app.world().entity(agent).get::<Transform>()
		);
	}

	#[test]
	fn face_translation_ignoring_height_difference() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(4., 5., 6.),
				_Face(Face::Translation(Vec3::new(10., 11., 12.))),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&Transform::from_xyz(4., 5., 6.).looking_at(Vec3::new(10., 5., 12.), Vec3::Y)),
			app.world().entity(agent).get::<Transform>()
		);
	}

	#[test]
	fn face_direction() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(4., 5., 6.),
				_Face(Face::Direction(Dir3::NEG_X)),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&Transform::from_xyz(4., 5., 6.).looking_to(Dir3::NEG_X, Vec3::Y)),
			app.world().entity(agent).get::<Transform>()
		);
	}

	#[test]
	fn do_not_default_rotation_when_looking_at_self_via_translation() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Transform::from_xyz(4., 5., 6.).looking_to(Vec3::new(1., 0., 1.), Vec3::Y),
				_Face(Face::Translation(Vec3::new(4., 5., 6.))),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&Transform::from_xyz(4., 5., 6.).looking_to(Vec3::new(1., 0., 1.), Vec3::Y)),
			app.world().entity(agent).get::<Transform>()
		);
	}

	#[test]
	fn do_not_default_rotation_when_looking_at_self_via_entity() {
		let mut app = setup();
		let persistent_agent = PersistentEntity::default();
		let agent = app
			.world_mut()
			.spawn((
				persistent_agent,
				Transform::from_xyz(4., 5., 6.).looking_to(Vec3::new(1., 0., 1.), Vec3::Y),
				_Face(Face::Entity(persistent_agent)),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&Transform::from_xyz(4., 5., 6.).looking_to(Vec3::new(1., 0., 1.), Vec3::Y)),
			app.world().entity(agent).get::<Transform>()
		);
	}
}
