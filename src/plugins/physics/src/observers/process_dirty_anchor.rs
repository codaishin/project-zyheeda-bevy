use crate::{
	components::anchor::{Anchor, AnchorDirty},
	system_params::mount_points_lookup::{MountPointsLookup, get_mount_point::MountPointError},
	traits::get_mount_point::GetMountPoint,
};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	components::persistent_entity::PersistentEntity,
	errors::{ErrorData, Level},
	traits::{
		accessors::get::{Get, TryApplyOn},
		handles_skill_physics::SkillSpawner,
	},
	zyheeda_commands::ZyheedaCommands,
};
use std::fmt::Display;

impl AnchorDirty {
	pub(crate) fn process(
		on_add: On<Add, Self>,
		commands: ZyheedaCommands,
		lookup: StaticSystemParam<MountPointsLookup<SkillSpawner>>,
		agents: Query<(&Anchor, &mut Transform)>,
		transforms: Query<&GlobalTransform>,
	) -> Result<(), AnchorError<MountPointError<SkillSpawner>>> {
		Self::system_internal(on_add, commands, lookup, agents, transforms)
	}

	fn system_internal<TLookup, TMountError>(
		on_add: On<Add, Self>,
		mut commands: ZyheedaCommands,
		mut lookup: StaticSystemParam<TLookup>,
		mut agents: Query<(&Anchor, &mut Transform)>,
		transforms: Query<&GlobalTransform>,
	) -> Result<(), AnchorError<TMountError>>
	where
		TLookup: for<'w, 's> SystemParam<
			Item<'w, 's>: GetMountPoint<SkillSpawner, TError = TMountError>,
		>,
	{
		let Ok((anchor, mut anchor_transform)) = agents.get_mut(on_add.entity) else {
			return Ok(());
		};

		commands.try_apply_on(&on_add.entity, |mut e| {
			if anchor.persistent {
				e.try_remove::<AnchorDirty>();
			} else {
				e.try_remove::<(AnchorDirty, Anchor)>();
			}
		});

		let Some(attached_to) = commands.get(&anchor.attached_to) else {
			return Err(AnchorError::AttachedToNotFound(anchor.attached_to));
		};

		let mount = match lookup.get_mount_point(attached_to, anchor.attach_point) {
			Ok(mount) => mount,
			Err(error) => return Err(AnchorError::MountError(error)),
		};

		let Ok(attached_to_transform) = transforms.get(attached_to) else {
			return Err(AnchorError::AttachedToNoTransform(anchor.attached_to));
		};

		let Ok(mount_transform) = transforms.get(mount) else {
			return Err(AnchorError::MountHasNoTransform(mount));
		};

		let mount_translation = mount_transform.translation();
		if mount_translation.is_nan() {
			return Err(AnchorError::MountTranslationIsNan(mount));
		}

		anchor_transform.translation = mount_translation;
		let rotation = match anchor.use_attached_rotation {
			true => attached_to_transform.rotation(),
			false => mount_transform.rotation(),
		};
		anchor_transform.rotation = rotation;

		Ok(())
	}
}

#[derive(Debug, PartialEq)]
pub(crate) enum AnchorError<TMountError> {
	MountError(TMountError),
	AttachedToNotFound(PersistentEntity),
	AttachedToNoTransform(PersistentEntity),
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
			AnchorError::AttachedToNotFound(e) => write!(f, "{e:?}: Anchor target not found"),
			AnchorError::AttachedToNoTransform(e) => write!(f, "{e:?}: Has no transform"),
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

	static AGENT: LazyLock<PersistentEntity> = LazyLock::new(PersistentEntity::default);

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(Result<(), AnchorError<_Error>>);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.register_persistent_entities();
		app.add_observer(
			AnchorDirty::system_internal::<ResMut<_Lookup>, _Error>.pipe(
				|In(result), mut commands: Commands| {
					commands.insert_resource(_Result(result));
				},
			),
		);
		app.insert_resource(_Lookup {
			mount_points: HashMap::default(),
		});

		app
	}

	#[test]
	fn copy_location_translation() {
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
		app.insert_resource(_Lookup {
			mount_points: HashMap::from([((spawner_key, agent), mount_point)]),
		});

		let anchor = app
			.world_mut()
			.spawn(Anchor::attach_to(*AGENT).on(spawner_key))
			.id();

		assert_eq!(
			Some(&Transform::from_xyz(4., 11., 9.)),
			app.world().entity(anchor).get::<Transform>()
		);
	}

	#[test]
	fn copy_location_rotation_of_mount_point() {
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
		app.insert_resource(_Lookup {
			mount_points: HashMap::from([((spawner_key, agent), mount_point)]),
		});

		let anchor = app
			.world_mut()
			.spawn(Anchor::attach_to(*AGENT).on(spawner_key));

		assert_eq!(
			Some(&Transform::from_xyz(4., 11., 9.).looking_to(Dir3::NEG_Z, Dir3::Y)),
			anchor.get::<Transform>(),
		);
	}

	#[test]
	fn copy_rotation_of_target() {
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
		app.insert_resource(_Lookup {
			mount_points: HashMap::from([((spawner_key, agent), mount_point)]),
		});

		let anchor = app.world_mut().spawn(
			Anchor::attach_to(*AGENT)
				.on(spawner_key)
				.with_attached_rotation(),
		);

		assert_eq!(
			Some(&Transform::from_xyz(4., 11., 9.).looking_to(Dir3::NEG_Z, Dir3::Y)),
			anchor.get::<Transform>(),
		);
	}

	#[test]
	fn do_not_change_scale() {
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
		app.insert_resource(_Lookup {
			mount_points: HashMap::from([((spawner_key, agent), mount_point)]),
		});

		let anchor = app
			.world_mut()
			.spawn(Anchor::attach_to(*AGENT).on(spawner_key));

		assert_eq!(
			Some(&Transform::from_xyz(4., 11., 9.)),
			anchor.get::<Transform>()
		);
	}

	#[test]
	fn remove_dirty_marker_on_one_time_anchor() {
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
		app.insert_resource(_Lookup {
			mount_points: HashMap::from([((spawner_key, agent), mount_point)]),
		});

		let anchor = app
			.world_mut()
			.spawn(Anchor::attach_to(*AGENT).on(spawner_key).once());

		assert!(!anchor.contains::<AnchorDirty>());
	}

	#[test]
	fn remove_dirty_marker_on_persistent_anchor() {
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
		app.insert_resource(_Lookup {
			mount_points: HashMap::from([((spawner_key, agent), mount_point)]),
		});

		let anchor = app
			.world_mut()
			.spawn(Anchor::attach_to(*AGENT).on(spawner_key).always());

		assert!(!anchor.contains::<AnchorDirty>());
	}

	#[test]
	fn remove_one_time_anchor() {
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
		app.insert_resource(_Lookup {
			mount_points: HashMap::from([((spawner_key, agent), mount_point)]),
		});

		let anchor = app
			.world_mut()
			.spawn(Anchor::attach_to(*AGENT).on(spawner_key).once());

		assert!(!anchor.contains::<Anchor>());
	}

	#[test]
	fn do_not_remove_persistent_anchor() {
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
		app.insert_resource(_Lookup {
			mount_points: HashMap::from([((spawner_key, agent), mount_point)]),
		});

		let anchor = app
			.world_mut()
			.spawn(Anchor::attach_to(*AGENT).on(spawner_key).always());

		assert!(anchor.contains::<Anchor>());
	}

	#[test]
	fn remove_components_in_error_case() {
		let mut app = setup();
		let spawner_key = SkillSpawner::Slot(SlotKey(22));

		let anchor = app
			.world_mut()
			.spawn(Anchor::attach_to(*AGENT).on(spawner_key).once());

		assert_eq!(
			(false, false),
			(
				anchor.contains::<Anchor>(),
				anchor.contains::<AnchorDirty>(),
			)
		);
	}

	#[test]
	fn return_target_error() {
		let mut app = setup();
		let spawner_key = SkillSpawner::Slot(SlotKey(22));

		app.world_mut()
			.spawn(Anchor::attach_to(*AGENT).on(spawner_key));

		assert_eq!(
			&_Result(Err(AnchorError::AttachedToNotFound(*AGENT))),
			app.world().resource::<_Result>(),
		);
	}

	#[test]
	fn return_mount_error() {
		let mut app = setup();
		let spawner_key = SkillSpawner::Slot(SlotKey(22));
		app.world_mut().spawn(*AGENT);

		app.world_mut()
			.spawn(Anchor::attach_to(*AGENT).on(spawner_key));

		assert_eq!(
			&_Result(Err(AnchorError::MountError(_Error))),
			app.world().resource::<_Result>(),
		);
	}

	#[test]
	fn return_root_no_transform_error() {
		let mut app = setup();
		let spawner_key = SkillSpawner::Slot(SlotKey(22));
		let agent = app.world_mut().spawn(*AGENT).id();
		let mount_point = app.world_mut().spawn_empty().id();
		app.insert_resource(_Lookup {
			mount_points: HashMap::from([((spawner_key, agent), mount_point)]),
		});

		app.world_mut()
			.spawn(Anchor::attach_to(*AGENT).on(spawner_key));

		assert_eq!(
			&_Result(Err(AnchorError::AttachedToNoTransform(*AGENT))),
			app.world().resource::<_Result>(),
		);
	}

	#[test]
	fn return_mount_point_no_transform_error() {
		let mut app = setup();
		let spawner_key = SkillSpawner::Slot(SlotKey(22));
		let agent = app
			.world_mut()
			.spawn((*AGENT, GlobalTransform::default()))
			.id();
		let mount_point = app.world_mut().spawn_empty().id();
		app.insert_resource(_Lookup {
			mount_points: HashMap::from([((spawner_key, agent), mount_point)]),
		});

		app.world_mut()
			.spawn(Anchor::attach_to(*AGENT).on(spawner_key));

		assert_eq!(
			&_Result(Err(AnchorError::MountHasNoTransform(mount_point))),
			app.world().resource::<_Result>(),
		);
	}

	#[test]
	fn return_mount_point_transform_nan_error() {
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
		app.insert_resource(_Lookup {
			mount_points: HashMap::from([((spawner_key, agent), mount_point)]),
		});

		app.world_mut()
			.spawn(Anchor::attach_to(*AGENT).on(spawner_key));

		assert_eq!(
			&_Result(Err(AnchorError::MountTranslationIsNan(mount_point))),
			app.world().resource::<_Result>(),
		);
	}
}
