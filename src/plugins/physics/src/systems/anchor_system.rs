use crate::{
	components::anchor::Anchor,
	system_params::mount_points_lookup::{MountPointsLookup, get_mount_point::MountPointError},
	traits::{get_mount_point::GetMountPoint, query_filter_definition::QueryFilterDefinition},
};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	components::persistent_entity::PersistentEntity,
	errors::{ErrorData, Level},
	traits::{accessors::get::Get, handles_skill_physics::SkillSpawner, or_ok::OrOk},
	zyheeda_commands::ZyheedaCommands,
};
use std::fmt::Display;

impl<TFilter> Anchor<TFilter>
where
	Self: QueryFilterDefinition + 'static,
{
	pub(crate) fn system(
		commands: ZyheedaCommands,
		lookup: StaticSystemParam<MountPointsLookup<SkillSpawner>>,
		agents: Query<(&Self, &mut Transform), <Self as QueryFilterDefinition>::TFilter>,
		transforms: Query<&GlobalTransform>,
	) -> Result<(), Vec<AnchorError<MountPointError<SkillSpawner>>>> {
		Self::system_internal(commands, lookup, agents, transforms)
	}

	fn system_internal<TLookup, TMountError>(
		commands: ZyheedaCommands,
		mut lookup: StaticSystemParam<TLookup>,
		mut agents: Query<(&Self, &mut Transform), <Self as QueryFilterDefinition>::TFilter>,
		transforms: Query<&GlobalTransform>,
	) -> Result<(), Vec<AnchorError<TMountError>>>
	where
		TLookup: for<'w, 's> SystemParam<
			Item<'w, 's>: GetMountPoint<SkillSpawner, TError = TMountError>,
		>,
	{
		agents
			.iter_mut()
			.filter_map(|(anchor, mut anchor_transform)| {
				let target = commands.get(&anchor.target)?;
				let mount_point = match lookup.get_mount_point(target, anchor.skill_spawner) {
					Ok(mount_point) => mount_point,
					Err(error) => return Some(AnchorError::MountError(error)),
				};

				let Ok(target_transform) = transforms.get(target) else {
					return Some(AnchorError::RootHasNoTransform(anchor.target));
				};

				let Ok(mount_point_transform) = transforms.get(mount_point) else {
					return Some(AnchorError::MountHasNoTransform(mount_point));
				};

				let mount_point_translation = mount_point_transform.translation();
				if mount_point_translation.is_nan() {
					return Some(AnchorError::MountTranslationIsNan(mount_point));
				}

				anchor_transform.translation = mount_point_translation;
				let rotation = match anchor.use_target_rotation {
					true => target_transform.rotation(),
					false => mount_point_transform.rotation(),
				};
				anchor_transform.rotation = rotation;

				None
			})
			.collect::<Vec<_>>()
			.or_ok(|| ())
	}
}

#[derive(Debug, PartialEq)]
pub(crate) enum AnchorError<TMountError> {
	MountError(TMountError),
	RootHasNoTransform(PersistentEntity),
	MountHasNoTransform(Entity),
	MountTranslationIsNan(Entity),
}

impl<TMountError> Display for AnchorError<TMountError>
where
	TMountError: Display,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			AnchorError::MountError(error) => write!(f, "{error}"),
			AnchorError::RootHasNoTransform(e) => write!(f, "{e:?}: Has no transform"),
			AnchorError::MountHasNoTransform(e) => write!(f, "{e}: Has no transform"),
			AnchorError::MountTranslationIsNan(e) => write!(f, "{e}: Translation is NaN"),
		}
	}
}

impl<TMountError> ErrorData for AnchorError<TMountError>
where
	TMountError: ErrorData + Display,
{
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl std::fmt::Display {
		"Anchor Error"
	}

	fn into_details(self) -> impl std::fmt::Display {
		self
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		components::persistent_entity::PersistentEntity,
		tools::action_key::slot::SlotKey,
		traits::{
			handles_skill_physics::SkillSpawner,
			register_persistent_entities::RegisterPersistentEntities,
		},
	};
	use std::{collections::HashMap, sync::LazyLock};
	use testing::SingleThreadedApp;

	struct _WithoutIgnore;

	#[derive(Component)]
	struct _Ignore;

	#[derive(Resource)]
	struct _Lookup {
		mount_points: HashMap<(SkillSpawner, Entity), Entity>,
	}

	impl GetMountPoint<SkillSpawner> for ResMut<'_, _Lookup> {
		type TError = _Error;

		fn get_mount_point(
			&mut self,
			root: Entity,
			key: SkillSpawner,
		) -> Result<Entity, Self::TError> {
			match self.mount_points.get(&(key, root)) {
				Some(entity) => Ok(*entity),
				None => Err(_Error),
			}
		}
	}

	#[derive(Debug, PartialEq)]
	struct _Error;

	impl QueryFilterDefinition for Anchor<_WithoutIgnore> {
		type TFilter = Without<_Ignore>;
	}

	static AGENT: LazyLock<PersistentEntity> = LazyLock::new(PersistentEntity::default);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.register_persistent_entities();
		app.insert_resource(_Lookup {
			mount_points: HashMap::default(),
		});

		app
	}

	#[test]
	fn copy_location_translation() -> Result<(), RunSystemError> {
		let spawner_key = SkillSpawner::Slot(SlotKey(22));
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((*AGENT, GlobalTransform::default()))
			.id();
		let mount_point = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(4., 11., 9.))
			.id();
		let anchor = app
			.world_mut()
			.spawn(Anchor::<_WithoutIgnore>::to_target(*AGENT).on_spawner(spawner_key))
			.id();
		app.insert_resource(_Lookup {
			mount_points: HashMap::from([((spawner_key, agent), mount_point)]),
		});

		_ = app.world_mut().run_system_once(
			Anchor::<_WithoutIgnore>::system_internal::<ResMut<_Lookup>, _Error>,
		)?;

		assert_eq!(
			Some(&Transform::from_xyz(4., 11., 9.)),
			app.world().entity(anchor).get::<Transform>()
		);
		Ok(())
	}

	#[test]
	fn copy_location_rotation_of_mount_point() -> Result<(), RunSystemError> {
		let mut app = setup();
		let spawner_key = SkillSpawner::Slot(SlotKey(22));
		let agent = app
			.world_mut()
			.spawn((*AGENT, GlobalTransform::default()))
			.id();
		let mount_point = app
			.world_mut()
			.spawn(GlobalTransform::from(
				Transform::from_xyz(4., 11., 9.).looking_to(Dir3::NEG_Z, Dir3::Y),
			))
			.id();
		let anchor = app
			.world_mut()
			.spawn(Anchor::<_WithoutIgnore>::to_target(*AGENT).on_spawner(spawner_key))
			.id();
		app.insert_resource(_Lookup {
			mount_points: HashMap::from([((spawner_key, agent), mount_point)]),
		});

		_ = app.world_mut().run_system_once(
			Anchor::<_WithoutIgnore>::system_internal::<ResMut<_Lookup>, _Error>,
		)?;

		assert_eq!(
			Some(&Transform::from_xyz(4., 11., 9.).looking_to(Dir3::NEG_Z, Dir3::Y)),
			app.world().entity(anchor).get::<Transform>(),
		);
		Ok(())
	}

	#[test]
	fn copy_rotation_of_target() -> Result<(), RunSystemError> {
		let mut app = setup();
		let spawner_key = SkillSpawner::Slot(SlotKey(22));
		let agent = app
			.world_mut()
			.spawn((
				*AGENT,
				GlobalTransform::from(Transform::default().looking_to(Dir3::NEG_Z, Dir3::Y)),
			))
			.id();
		let mount_point = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(4., 11., 9.))
			.id();
		let anchor = app
			.world_mut()
			.spawn(
				Anchor::<_WithoutIgnore>::to_target(*AGENT)
					.on_spawner(spawner_key)
					.with_target_rotation(),
			)
			.id();
		app.insert_resource(_Lookup {
			mount_points: HashMap::from([((spawner_key, agent), mount_point)]),
		});

		_ = app.world_mut().run_system_once(
			Anchor::<_WithoutIgnore>::system_internal::<ResMut<_Lookup>, _Error>,
		)?;

		assert_eq!(
			Some(&Transform::from_xyz(4., 11., 9.).looking_to(Dir3::NEG_Z, Dir3::Y)),
			app.world().entity(anchor).get::<Transform>(),
		);
		Ok(())
	}

	#[test]
	fn do_not_change_scale() -> Result<(), RunSystemError> {
		let mut app = setup();
		let spawner_key = SkillSpawner::Slot(SlotKey(22));
		let agent = app
			.world_mut()
			.spawn((
				*AGENT,
				GlobalTransform::from(Transform::default().with_scale(Vec3::splat(2.))),
			))
			.id();
		let mount_point = app
			.world_mut()
			.spawn(GlobalTransform::from(
				Transform::from_xyz(4., 11., 9.).with_scale(Vec3::splat(2.)),
			))
			.id();
		let anchor = app
			.world_mut()
			.spawn(Anchor::<_WithoutIgnore>::to_target(*AGENT).on_spawner(spawner_key))
			.id();
		app.insert_resource(_Lookup {
			mount_points: HashMap::from([((spawner_key, agent), mount_point)]),
		});

		_ = app.world_mut().run_system_once(
			Anchor::<_WithoutIgnore>::system_internal::<ResMut<_Lookup>, _Error>,
		)?;

		assert_eq!(
			Some(&Transform::from_xyz(4., 11., 9.)),
			app.world().entity(anchor).get::<Transform>()
		);
		Ok(())
	}

	#[test]
	fn apply_filter() -> Result<(), RunSystemError> {
		let mut app = setup();
		let spawner_key = SkillSpawner::Slot(SlotKey(22));
		let agent = app
			.world_mut()
			.spawn((*AGENT, GlobalTransform::default()))
			.id();
		let mount_point = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(4., 11., 9.))
			.id();
		let anchor = app
			.world_mut()
			.spawn((
				Anchor::<_WithoutIgnore>::to_target(*AGENT).on_spawner(spawner_key),
				_Ignore,
			))
			.id();
		app.insert_resource(_Lookup {
			mount_points: HashMap::from([((spawner_key, agent), mount_point)]),
		});

		_ = app.world_mut().run_system_once(
			Anchor::<_WithoutIgnore>::system_internal::<ResMut<_Lookup>, _Error>,
		)?;

		assert_eq!(
			Some(&Transform::default()),
			app.world().entity(anchor).get::<Transform>()
		);
		Ok(())
	}

	#[test]
	fn return_mount_error() -> Result<(), RunSystemError> {
		let mut app = setup();
		let spawner_key = SkillSpawner::Slot(SlotKey(22));
		app.world_mut().spawn(*AGENT);
		app.world_mut()
			.spawn(Anchor::<_WithoutIgnore>::to_target(*AGENT).on_spawner(spawner_key));

		let errors = app.world_mut().run_system_once(
			Anchor::<_WithoutIgnore>::system_internal::<ResMut<_Lookup>, _Error>,
		)?;

		assert_eq!(Err(vec![AnchorError::MountError(_Error)]), errors);
		Ok(())
	}

	#[test]
	fn return_root_no_transform_error() -> Result<(), RunSystemError> {
		let mut app = setup();
		let spawner_key = SkillSpawner::Slot(SlotKey(22));
		let agent = app.world_mut().spawn(*AGENT).id();
		let mount_point = app.world_mut().spawn_empty().id();
		app.world_mut()
			.spawn(Anchor::<_WithoutIgnore>::to_target(*AGENT).on_spawner(spawner_key));
		app.insert_resource(_Lookup {
			mount_points: HashMap::from([((spawner_key, agent), mount_point)]),
		});

		let errors = app.world_mut().run_system_once(
			Anchor::<_WithoutIgnore>::system_internal::<ResMut<_Lookup>, _Error>,
		)?;

		assert_eq!(Err(vec![AnchorError::RootHasNoTransform(*AGENT)]), errors);
		Ok(())
	}

	#[test]
	fn return_mount_point_no_transform_error() -> Result<(), RunSystemError> {
		let mut app = setup();
		let spawner_key = SkillSpawner::Slot(SlotKey(22));
		let agent = app
			.world_mut()
			.spawn((*AGENT, GlobalTransform::default()))
			.id();
		let mount_point = app.world_mut().spawn_empty().id();
		app.world_mut()
			.spawn(Anchor::<_WithoutIgnore>::to_target(*AGENT).on_spawner(spawner_key));
		app.insert_resource(_Lookup {
			mount_points: HashMap::from([((spawner_key, agent), mount_point)]),
		});

		let errors = app.world_mut().run_system_once(
			Anchor::<_WithoutIgnore>::system_internal::<ResMut<_Lookup>, _Error>,
		)?;

		assert_eq!(
			Err(vec![AnchorError::MountHasNoTransform(mount_point)]),
			errors
		);
		Ok(())
	}

	#[test]
	fn return_mount_point_transform_nan_error() -> Result<(), RunSystemError> {
		let mut app = setup();
		let spawner_key = SkillSpawner::Slot(SlotKey(22));
		let agent = app
			.world_mut()
			.spawn((*AGENT, GlobalTransform::default()))
			.id();
		let mount_point = app
			.world_mut()
			.spawn(GlobalTransform::from_translation(Vec3::NAN))
			.id();
		app.world_mut()
			.spawn(Anchor::<_WithoutIgnore>::to_target(*AGENT).on_spawner(spawner_key));
		app.insert_resource(_Lookup {
			mount_points: HashMap::from([((spawner_key, agent), mount_point)]),
		});

		let errors = app.world_mut().run_system_once(
			Anchor::<_WithoutIgnore>::system_internal::<ResMut<_Lookup>, _Error>,
		)?;

		assert_eq!(
			Err(vec![AnchorError::MountTranslationIsNan(mount_point)]),
			errors
		);
		Ok(())
	}
}
