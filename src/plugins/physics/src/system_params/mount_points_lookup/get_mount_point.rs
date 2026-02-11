use crate::{
	components::mount_points::MountPointsDefinition,
	system_params::mount_points_lookup::MountPointsLookup,
	traits::get_mount_point::GetMountPoint,
};
use bevy::prelude::*;
use common::{
	errors::{ErrorData, Level},
	traits::thread_safe::ThreadSafe,
};
use std::{
	any::type_name,
	fmt::{Debug, Display},
	hash::Hash,
};

impl<T> GetMountPoint<T> for MountPointsLookup<'_, '_, T>
where
	T: ThreadSafe + Eq + Hash,
{
	type TError = MountPointError<T>;

	fn get_mount_point(&mut self, entity: Entity, key: T) -> Result<Entity, Self::TError> {
		let Ok((MountPointsDefinition(def), mut cache)) = self.mount_points.get_mut(entity) else {
			return Err(MountPointError::CacheOrDefinitionMissing { entity });
		};

		if let Some(bone) = cache.0.get(&key) {
			return Ok(*bone);
		}

		for child in self.children.iter_descendants(entity) {
			let Ok(name) = self.names.get(child) else {
				continue;
			};

			let Some(key_def) = def.get(name.as_str()) else {
				continue;
			};

			if key_def != &key {
				continue;
			}

			cache.0.insert(key, child);
			return Ok(child);
		}

		Err(MountPointError::NoMountPointFoundFor { entity, key })
	}
}

#[derive(Debug, PartialEq)]
pub(crate) enum MountPointError<T> {
	CacheOrDefinitionMissing { entity: Entity },
	NoMountPointFoundFor { entity: Entity, key: T },
}

impl<T> Display for MountPointError<T>
where
	T: Debug,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			MountPointError::CacheOrDefinitionMissing { entity } => {
				let key_name = type_name::<T>();
				write!(f, "{entity}: No mount points definition for {key_name}")
			}
			MountPointError::NoMountPointFoundFor { entity, key } => {
				write!(f, "{entity}: No mount point found for ({key:?})")
			}
		}
	}
}

impl<T> ErrorData for MountPointError<T>
where
	T: Debug,
{
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl std::fmt::Display {
		"Mount Point Error"
	}

	fn into_details(self) -> impl std::fmt::Display {
		self
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::mount_points::MountPoints;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::tools::bone_name::BoneName;
	use std::collections::HashMap;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	enum _Key {
		A,
		B,
	}

	#[test]
	fn return_matching_entity() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(MountPointsDefinition(HashMap::from([
				(BoneName::from("a"), _Key::A),
				(BoneName::from("b"), _Key::B),
			])))
			.id();
		let bone = app
			.world_mut()
			.spawn((ChildOf(entity), Name::from("a")))
			.id();

		let mount_point =
			app.world_mut()
				.run_system_once(move |mut lookup: MountPointsLookup<_Key>| {
					lookup.get_mount_point(entity, _Key::A)
				})?;

		assert_eq!(Ok(bone), mount_point);
		Ok(())
	}

	#[test]
	fn return_matching_entity_with_other_key() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(MountPointsDefinition(HashMap::from([
				(BoneName::from("a"), _Key::A),
				(BoneName::from("b"), _Key::B),
			])))
			.id();
		app.world_mut().spawn((ChildOf(entity), Name::from("a")));
		let bone = app
			.world_mut()
			.spawn((ChildOf(entity), Name::from("b")))
			.id();

		let mount_point =
			app.world_mut()
				.run_system_once(move |mut lookup: MountPointsLookup<_Key>| {
					lookup.get_mount_point(entity, _Key::B)
				})?;

		assert_eq!(Ok(bone), mount_point);
		Ok(())
	}

	#[test]
	fn store_found_entity() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(MountPointsDefinition(HashMap::from([
				(BoneName::from("a"), _Key::A),
				(BoneName::from("b"), _Key::B),
			])))
			.id();
		let bone = app
			.world_mut()
			.spawn((ChildOf(entity), Name::from("a")))
			.id();

		app.world_mut()
			.run_system_once(move |mut lookup: MountPointsLookup<_Key>| {
				_ = lookup.get_mount_point(entity, _Key::A);
			})?;

		assert_eq!(
			Some(&MountPoints(HashMap::from([(_Key::A, bone)]))),
			app.world().entity(entity).get::<MountPoints<_Key>>()
		);
		Ok(())
	}

	#[test]
	fn use_cached_entity() -> Result<(), RunSystemError> {
		let mut app = setup();
		let bone = app.world_mut().spawn_empty().id();
		let entity = app
			.world_mut()
			.spawn((
				MountPointsDefinition::<_Key>(HashMap::from([])),
				MountPoints(HashMap::from([(_Key::A, bone)])),
			))
			.id();

		let mount_point =
			app.world_mut()
				.run_system_once(move |mut lookup: MountPointsLookup<_Key>| {
					lookup.get_mount_point(entity, _Key::A)
				})?;

		assert_eq!(Ok(bone), mount_point);
		Ok(())
	}

	#[test]
	fn prioritize_cached_value() -> Result<(), RunSystemError> {
		// This test does not represent an expected scenario.
		// We just need to see that the cached value is preferred over the current tree state.
		let mut app = setup();
		let bone = app.world_mut().spawn_empty().id();
		let entity = app
			.world_mut()
			.spawn((
				MountPointsDefinition(HashMap::from([(BoneName::from("a"), _Key::A)])),
				MountPoints(HashMap::from([(_Key::A, bone)])),
			))
			.id();
		app.world_mut().spawn((ChildOf(entity), Name::from("a")));

		let mount_point =
			app.world_mut()
				.run_system_once(move |mut lookup: MountPointsLookup<_Key>| {
					lookup.get_mount_point(entity, _Key::A)
				})?;

		assert_eq!(Ok(bone), mount_point);
		Ok(())
	}

	#[test]
	fn return_cache_or_definition_missing() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		let mount_point =
			app.world_mut()
				.run_system_once(move |mut lookup: MountPointsLookup<_Key>| {
					lookup.get_mount_point(entity, _Key::A)
				})?;

		assert_eq!(
			Err(MountPointError::CacheOrDefinitionMissing { entity }),
			mount_point
		);
		Ok(())
	}

	#[test]
	fn return_no_mount_point_found() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(MountPointsDefinition::<_Key>::default())
			.id();

		let mount_point =
			app.world_mut()
				.run_system_once(move |mut lookup: MountPointsLookup<_Key>| {
					lookup.get_mount_point(entity, _Key::A)
				})?;

		assert_eq!(
			Err(MountPointError::NoMountPointFoundFor {
				entity,
				key: _Key::A
			}),
			mount_point
		);
		Ok(())
	}
}
