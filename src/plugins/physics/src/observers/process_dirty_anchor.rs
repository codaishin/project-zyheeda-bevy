use crate::{
	components::anchor::{Anchor, AnchorDirty, AnchorRotation},
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
		handles_physics::{MouseHover, MouseHoversOver, Raycast},
		handles_skill_physics::{SkillSpawner, SkillTarget},
	},
	zyheeda_commands::ZyheedaCommands,
};
use std::fmt::Display;

impl AnchorDirty {
	pub(crate) fn process<TRayCaster>(
		on_add: On<Add, Self>,
		commands: ZyheedaCommands,
		lookup: StaticSystemParam<MountPointsLookup<SkillSpawner>>,
		ray_caster: StaticSystemParam<TRayCaster>,
		agents: Query<(&Anchor, &mut Transform)>,
		transforms: Query<&GlobalTransform>,
	) -> Result<(), AnchorError<MountPointError<SkillSpawner>>>
	where
		TRayCaster: for<'w, 's> SystemParam<Item<'w, 's>: Raycast<MouseHover>>,
	{
		Self::system_internal(on_add, commands, lookup, ray_caster, agents, transforms)
	}

	fn system_internal<TLookup, TRayCaster, TMountError>(
		on_add: On<Add, Self>,
		mut commands: ZyheedaCommands,
		mut lookup: StaticSystemParam<TLookup>,
		mut ray_caster: StaticSystemParam<TRayCaster>,
		mut agents: Query<(&Anchor, &mut Transform)>,
		transforms: Query<&GlobalTransform>,
	) -> Result<(), AnchorError<TMountError>>
	where
		TLookup: for<'w, 's> SystemParam<
			Item<'w, 's>: GetMountPoint<SkillSpawner, TError = TMountError>,
		>,
		TRayCaster: for<'w, 's> SystemParam<Item<'w, 's>: Raycast<MouseHover>>,
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
			return Err(AnchorError::EntityNotFound(anchor.attached_to));
		};

		let mount = match lookup.get_mount_point(attached_to, anchor.attach_point) {
			Ok(mount) => mount,
			Err(error) => return Err(AnchorError::MountError(error)),
		};

		let Ok(attached_to_transform) = transforms.get(attached_to) else {
			return Err(AnchorError::PersistentEntityWithoutTransform(
				anchor.attached_to,
			));
		};

		let Ok(mount_transform) = transforms.get(mount) else {
			return Err(AnchorError::EntityWithoutTransform(mount));
		};

		let mount_translation = mount_transform.translation();
		if mount_translation.is_nan() {
			return Err(AnchorError::TranslationNaN(mount));
		}

		anchor_transform.translation = mount_translation;
		match anchor.rotation {
			AnchorRotation::OfAttachedTo => {
				anchor_transform.rotation = attached_to_transform.rotation();
			}
			AnchorRotation::OfMount => {
				anchor_transform.rotation = mount_transform.rotation();
			}
			AnchorRotation::LookingAt(SkillTarget::Cursor) => {
				let hover = MouseHover {
					exclude: vec![attached_to],
				};
				let Some(hit) = ray_caster.raycast(hover) else {
					return Ok(());
				};
				let target = match hit {
					MouseHoversOver::Terrain { point } => point,
					MouseHoversOver::Object { entity, .. } => {
						let Ok(target) = transforms.get(entity) else {
							return Err(AnchorError::EntityWithoutTransform(entity));
						};

						target.translation()
					}
				};

				anchor_transform.look_at(target, Vec3::Y);
			}
			AnchorRotation::LookingAt(SkillTarget::Entity(entity)) => {
				let Some(target) = commands.get(&entity) else {
					return Err(AnchorError::EntityNotFound(entity));
				};
				let Ok(target) = transforms.get(target) else {
					return Err(AnchorError::EntityWithoutTransform(target));
				};

				anchor_transform.look_at(target.translation(), Vec3::Y);
			}
		};

		Ok(())
	}
}

#[derive(Debug, PartialEq)]
pub(crate) enum AnchorError<TMountError> {
	MountError(TMountError),
	EntityNotFound(PersistentEntity),
	PersistentEntityWithoutTransform(PersistentEntity),
	EntityWithoutTransform(Entity),
	TranslationNaN(Entity),
}

impl<TMountError> Display for AnchorError<TMountError>
where
	TMountError: Display,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			AnchorError::MountError(error) => write!(f, "{error}"),
			AnchorError::EntityNotFound(e) => write!(f, "{e:?}: not found"),
			AnchorError::PersistentEntityWithoutTransform(e) => {
				write!(f, "{e:?}: has no transform")
			}
			AnchorError::EntityWithoutTransform(e) => write!(f, "{e}: has no transform"),
			AnchorError::TranslationNaN(e) => write!(f, "{e}: translation is NaN"),
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
			handles_physics::MouseHoversOver,
			handles_skill_physics::SkillSpawner,
			register_persistent_entities::RegisterPersistentEntities,
		},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::{collections::HashMap, sync::LazyLock};
	use testing::{NestedMocks, SingleThreadedApp};

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

	#[derive(Resource, NestedMocks)]
	struct _RayCaster {
		mock: Mock_RayCaster,
	}

	impl Default for _RayCaster {
		fn default() -> Self {
			Self::new().with_mock(|mock| {
				mock.expect_raycast().return_const(None);
			})
		}
	}

	#[automock]
	impl Raycast<MouseHover> for _RayCaster {
		fn raycast(&mut self, args: MouseHover) -> Option<MouseHoversOver> {
			self.mock.raycast(args)
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
			AnchorDirty::system_internal::<ResMut<_Lookup>, ResMut<_RayCaster>, _Error>.pipe(
				|In(result), mut commands: Commands| {
					commands.insert_resource(_Result(result));
				},
			),
		);
		app.init_resource::<_RayCaster>();
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
	fn copy_rotation_of_anchor() {
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
	fn look_at_target() {
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
		let target = PersistentEntity::default();
		app.world_mut()
			.spawn((target, GlobalTransform::from_xyz(11., -20., 3.)));

		let anchor = app.world_mut().spawn(
			Anchor::attach_to(*AGENT)
				.on(spawner_key)
				.looking_at(SkillTarget::Entity(target)),
		);

		assert_eq!(
			Some(&Transform::from_xyz(4., 11., 9.).looking_at(Vec3::new(11., -20., 3.), Vec3::Y)),
			anchor.get::<Transform>(),
		);
	}

	#[test]
	fn look_at_cursor_over_terrain() {
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
		app.insert_resource(_RayCaster::new().with_mock(|mock| {
			mock.expect_raycast()
				.once()
				.with(eq(MouseHover {
					exclude: vec![agent],
				}))
				.return_const(MouseHoversOver::Terrain {
					point: Vec3::new(11., 22., 33.),
				});
		}));

		let anchor = app.world_mut().spawn(
			Anchor::attach_to(*AGENT)
				.on(spawner_key)
				.looking_at(SkillTarget::Cursor),
		);

		assert_eq!(
			Some(&Transform::from_xyz(4., 11., 9.).looking_at(Vec3::new(11., 22., 33.), Vec3::Y)),
			anchor.get::<Transform>(),
		);
	}

	#[test]
	fn look_at_cursor_over_object() {
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
		let target = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(11., 22., 33.))
			.id();
		app.insert_resource(_RayCaster::new().with_mock(|mock| {
			mock.expect_raycast()
				.once()
				.with(eq(MouseHover {
					exclude: vec![agent],
				}))
				.return_const(MouseHoversOver::Object {
					entity: target,
					point: Vec3::ZERO,
				});
		}));

		let anchor = app.world_mut().spawn(
			Anchor::attach_to(*AGENT)
				.on(spawner_key)
				.looking_at(SkillTarget::Cursor),
		);

		assert_eq!(
			Some(&Transform::from_xyz(4., 11., 9.).looking_at(Vec3::new(11., 22., 33.), Vec3::Y)),
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
			&_Result(Err(AnchorError::EntityNotFound(*AGENT))),
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
			&_Result(Err(AnchorError::PersistentEntityWithoutTransform(*AGENT))),
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
			&_Result(Err(AnchorError::EntityWithoutTransform(mount_point))),
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
			&_Result(Err(AnchorError::TranslationNaN(mount_point))),
			app.world().resource::<_Result>(),
		);
	}
}
