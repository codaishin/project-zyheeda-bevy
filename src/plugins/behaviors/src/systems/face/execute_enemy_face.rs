use crate::components::Attack;
use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	traits::{accessors::get::GetMut, handles_orientation::Face},
	zyheeda_commands::ZyheedaCommands,
};

pub(crate) fn execute_enemy_face(
	In(faces): In<Vec<(Entity, Face)>>,
	mut commands: ZyheedaCommands,
	mut transforms: Query<&mut Transform>,
	attacks: Query<&Attack>,
) {
	for (entity, face) in faces {
		let (mut transform, target) = match face {
			Face::Target => {
				let Ok(Attack(target)) = attacks.get(entity) else {
					continue;
				};
				let Some(target) = get_translation(&mut commands, &transforms, target) else {
					continue;
				};
				let Ok(transform) = transforms.get_mut(entity) else {
					continue;
				};
				(transform, target)
			}
			Face::Entity(target) => {
				let Some(target) = get_translation(&mut commands, &transforms, &target) else {
					continue;
				};
				let Ok(transform) = transforms.get_mut(entity) else {
					continue;
				};
				(transform, target)
			}
			Face::Translation(target) => {
				let Ok(transform) = transforms.get_mut(entity) else {
					continue;
				};
				(transform, target)
			}
		};

		if transform.translation == target {
			continue;
		}

		transform.look_at(target, Vec3::Y);
	}
}

fn get_translation(
	commands: &mut ZyheedaCommands,
	transforms: &Query<&mut Transform>,
	target: &PersistentEntity,
) -> Option<Vec3> {
	commands
		.get_mut(target)
		.and_then(|target| transforms.get(target.id()).ok())
		.map(|target| target.translation)
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		components::persistent_entity::PersistentEntity,
		traits::register_persistent_entities::RegisterPersistentEntities,
	};
	use std::sync::LazyLock;
	use testing::SingleThreadedApp;

	static TARGET: LazyLock<PersistentEntity> = LazyLock::new(PersistentEntity::default);

	#[derive(Component)]
	#[require(PersistentEntity = *TARGET)]
	struct _Target;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.register_persistent_entities();

		app
	}

	mod target {
		use super::*;

		#[test]
		fn face() -> Result<(), RunSystemError> {
			let mut app = setup();
			app.world_mut()
				.spawn((Transform::from_xyz(1., 2., 3.), _Target));
			let agent = app
				.world_mut()
				.spawn((Transform::from_xyz(5., 4., 11.), Attack(*TARGET)))
				.id();

			app.world_mut()
				.run_system_once_with(execute_enemy_face, vec![(agent, Face::Target)])?;

			assert_eq!(
				Some(&Transform::from_xyz(5., 4., 11.).looking_at(Vec3::new(1., 2., 3.), Vec3::Y)),
				app.world().entity(agent).get::<Transform>()
			);
			Ok(())
		}

		#[test]
		fn no_default_rotation_when_looking_at_self() -> Result<(), RunSystemError> {
			let mut app = setup();
			let persistent_agent = PersistentEntity::default();
			let agent = app
				.world_mut()
				.spawn((
					persistent_agent,
					Transform::from_xyz(5., 4., 11.).looking_to(Vec3::X, Vec3::Y),
					Attack(persistent_agent),
				))
				.id();

			app.world_mut()
				.run_system_once_with(execute_enemy_face, vec![(agent, Face::Target)])?;

			assert_eq!(
				Some(&Transform::from_xyz(5., 4., 11.).looking_to(Vec3::X, Vec3::Y)),
				app.world().entity(agent).get::<Transform>()
			);
			Ok(())
		}
	}

	mod translation {
		use super::*;

		#[test]
		fn face() -> Result<(), RunSystemError> {
			let mut app = setup();
			let agent = app.world_mut().spawn(Transform::from_xyz(4., 5., 6.)).id();

			app.world_mut().run_system_once_with(
				execute_enemy_face,
				vec![(agent, Face::Translation(Vec3::new(10., 11., 12.)))],
			)?;

			assert_eq!(
				Some(
					&Transform::from_xyz(4., 5., 6.).looking_at(Vec3::new(10., 11., 12.), Vec3::Y)
				),
				app.world().entity(agent).get::<Transform>()
			);
			Ok(())
		}

		#[test]
		fn no_default_rotation_when_looking_at_self() -> Result<(), RunSystemError> {
			let mut app = setup();
			let persistent_agent = PersistentEntity::default();
			let agent = app
				.world_mut()
				.spawn((
					persistent_agent,
					Transform::from_xyz(4., 5., 6.).looking_to(Vec3::X, Vec3::Y),
					Attack(persistent_agent),
				))
				.id();

			app.world_mut().run_system_once_with(
				execute_enemy_face,
				vec![(agent, Face::Translation(Vec3::new(4., 5., 6.)))],
			)?;

			assert_eq!(
				Some(&Transform::from_xyz(4., 5., 6.).looking_to(Vec3::X, Vec3::Y)),
				app.world().entity(agent).get::<Transform>()
			);
			Ok(())
		}
	}

	mod entity {
		use super::*;

		#[test]
		fn face() -> Result<(), RunSystemError> {
			let mut app = setup();
			app.world_mut()
				.spawn((Transform::from_xyz(15., 2., 3.), _Target));
			let agent = app.world_mut().spawn(Transform::from_xyz(1., 4., 11.)).id();

			app.world_mut()
				.run_system_once_with(execute_enemy_face, vec![(agent, Face::Entity(*TARGET))])?;

			assert_eq!(
				Some(&Transform::from_xyz(1., 4., 11.).looking_at(Vec3::new(15., 2., 3.), Vec3::Y)),
				app.world().entity(agent).get::<Transform>()
			);
			Ok(())
		}

		#[test]
		fn no_default_rotation_when_looking_at_self() -> Result<(), RunSystemError> {
			let mut app = setup();
			let persistent_agent = PersistentEntity::default();
			let agent = app
				.world_mut()
				.spawn((
					persistent_agent,
					Transform::from_xyz(1., 4., 11.).looking_to(Vec3::X, Vec3::Y),
					Attack(persistent_agent),
				))
				.id();

			app.world_mut().run_system_once_with(
				execute_enemy_face,
				vec![(agent, Face::Entity(persistent_agent))],
			)?;

			assert_eq!(
				Some(&Transform::from_xyz(1., 4., 11.).looking_to(Vec3::X, Vec3::Y)),
				app.world().entity(agent).get::<Transform>()
			);
			Ok(())
		}
	}
}
