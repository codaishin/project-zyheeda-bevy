pub(crate) mod spawner_fix_point;

use super::{Always, Once};
use crate::{
	components::anchor::spawner_fix_point::SpawnerFixPoint,
	traits::has_filter::HasFilter,
};
use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	errors::{Error, Level},
	resources::persistent_entities::PersistentEntities,
	traits::track::{IsTracking, Track, Untrack},
};
use std::{
	any::{TypeId, type_name},
	collections::HashMap,
	marker::PhantomData,
};

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub(crate) struct Anchor<TFilter> {
	pub(crate) target: PersistentEntity,
	pub(crate) fix_point_key: AnchorFixPointKey,
	_p: PhantomData<TFilter>,
}

impl HasFilter for Anchor<Once> {
	type TFilter = Added<Self>;
}

impl HasFilter for Anchor<Always> {
	type TFilter = ();
}

impl<TFilter> Anchor<TFilter>
where
	Self: HasFilter + Send + Sync + 'static,
{
	pub(crate) fn to(target: PersistentEntity) -> AnchorBuilder<TFilter> {
		AnchorBuilder {
			target,
			_p: PhantomData,
		}
	}

	pub(crate) fn system(
		mut persistent_entities: ResMut<PersistentEntities>,
		mut agents: Query<(&Self, &mut Transform), <Self as HasFilter>::TFilter>,
		fix_points: Query<&AnchorFixPoints>,
		transforms: Query<&GlobalTransform>,
	) -> Vec<Result<(), AnchorError>> {
		agents
			.iter_mut()
			.filter_map(|(anchor, mut anchor_transform)| {
				let target = persistent_entities.get_entity(&anchor.target)?;
				let Ok(AnchorFixPoints(fix_points)) = fix_points.get(target) else {
					return Some(AnchorError::FixPointsMissingOn(anchor.target));
				};
				let Some(fix_point) = fix_points.get(&anchor.fix_point_key).copied() else {
					return Some(AnchorError::NoFixPointEntityFor(anchor.fix_point_key));
				};
				let Ok(fix_point) = transforms.get(fix_point) else {
					return Some(AnchorError::GlobalTransformMissingOn(fix_point));
				};

				let fix_point = Transform::from(*fix_point);

				anchor_transform.translation = fix_point.translation;
				anchor_transform.rotation = fix_point.rotation;

				None
			})
			.map(Err)
			.collect()
	}
}

pub(crate) struct AnchorBuilder<TFilter> {
	target: PersistentEntity,
	_p: PhantomData<TFilter>,
}

impl<TFilter> AnchorBuilder<TFilter> {
	pub(crate) fn on_fix_point<TFixPoint>(self, fix_point_key: TFixPoint) -> Anchor<TFilter>
	where
		TFixPoint: Into<AnchorFixPointKey>,
	{
		Anchor {
			target: self.target,
			fix_point_key: fix_point_key.into(),
			_p: PhantomData,
		}
	}
}

#[derive(Component, Debug, PartialEq, Clone, Default)]
pub struct AnchorFixPoints(HashMap<AnchorFixPointKey, Entity>);

impl Track<SpawnerFixPoint> for AnchorFixPoints {
	fn track(&mut self, entity: Entity, spawner_fix_point: &SpawnerFixPoint) {
		self.0.insert((*spawner_fix_point).into(), entity);
	}
}

impl Untrack<SpawnerFixPoint> for AnchorFixPoints {
	fn untrack(&mut self, entity: &Entity) {
		self.0
			.retain(|k, e| e != entity || k.source_type != TypeId::of::<SpawnerFixPoint>());
	}
}

impl IsTracking<SpawnerFixPoint> for AnchorFixPoints {
	fn is_tracking(&self, entity: &Entity) -> bool {
		self.0
			.iter()
			.any(|(k, e)| e == entity && k.source_type == TypeId::of::<SpawnerFixPoint>())
	}
}

impl<const N: usize, TKey> From<[(TKey, Entity); N]> for AnchorFixPoints
where
	TKey: Into<AnchorFixPointKey>,
{
	fn from(fix_points: [(TKey, Entity); N]) -> Self {
		Self(HashMap::from(
			fix_points.map(|(key, entity)| (key.into(), entity)),
		))
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct AnchorFixPointKey {
	key: usize,
	source_type: TypeId,
}

impl AnchorFixPointKey {
	fn new<TSource>(key: usize) -> Self
	where
		TSource: 'static,
	{
		Self {
			key,
			source_type: TypeId::of::<TSource>(),
		}
	}
}

#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum AnchorError {
	NoFixPointEntityFor(AnchorFixPointKey),
	FixPointsMissingOn(PersistentEntity),
	GlobalTransformMissingOn(Entity),
}

impl From<AnchorError> for Error {
	fn from(error: AnchorError) -> Self {
		match error {
			AnchorError::FixPointsMissingOn(entity) => {
				let type_name = type_name::<AnchorFixPoints>();
				Self {
					msg: format!("{entity:?}: {type_name} missing"),
					lvl: Level::Error,
				}
			}
			AnchorError::GlobalTransformMissingOn(entity) => {
				let type_name = type_name::<GlobalTransform>();
				Self {
					msg: format!("{entity}: {type_name} missing"),
					lvl: Level::Error,
				}
			}
			AnchorError::NoFixPointEntityFor(anchor_fix_point_key) => Self {
				msg: format!("{anchor_fix_point_key:?} missing"),
				lvl: Level::Error,
			},
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::traits::{
		handles_skill_behaviors::Spawner,
		register_persistent_entities::RegisterPersistentEntities,
	};
	use testing::SingleThreadedApp;

	struct _NotIgnored;

	#[derive(Component)]
	struct _Ignore;

	impl HasFilter for Anchor<_NotIgnored> {
		type TFilter = Without<_Ignore>;
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.register_persistent_entities();

		app
	}

	#[test]
	fn copy_location_translation() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = PersistentEntity::default();
		let fix_point = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(4., 11., 9.))
			.id();
		app.world_mut().spawn((
			AnchorFixPoints::from([(AnchorFixPointKey::new::<()>(11), fix_point)]),
			entity,
		));
		let agent = app
			.world_mut()
			.spawn((
				Anchor::<_NotIgnored>::to(entity).on_fix_point(AnchorFixPointKey::new::<()>(11)),
				Transform::default(),
			))
			.id();

		_ = app
			.world_mut()
			.run_system_once(Anchor::<_NotIgnored>::system)?;

		assert_eq!(
			Some(&Transform::from_xyz(4., 11., 9.)),
			app.world().entity(agent).get::<Transform>()
		);
		Ok(())
	}

	#[test]
	fn copy_location_rotation() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = PersistentEntity::default();
		let fix_point = app
			.world_mut()
			.spawn(GlobalTransform::from(
				Transform::default().looking_at(Vec3::new(0., 0., 1.), Vec3::Y),
			))
			.id();
		app.world_mut().spawn((
			AnchorFixPoints::from([(AnchorFixPointKey::new::<()>(11), fix_point)]),
			entity,
		));
		let agent = app
			.world_mut()
			.spawn((
				Anchor::<_NotIgnored>::to(entity).on_fix_point(AnchorFixPointKey::new::<()>(11)),
				Transform::default(),
			))
			.id();

		_ = app
			.world_mut()
			.run_system_once(Anchor::<_NotIgnored>::system)?;

		assert_eq!(
			Some(&Transform::default().looking_at(Vec3::new(0., 0., 1.), Vec3::Y)),
			app.world().entity(agent).get::<Transform>()
		);
		Ok(())
	}

	#[test]
	fn do_not_change_scale() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = PersistentEntity::default();
		let fix_point = app
			.world_mut()
			.spawn(GlobalTransform::from(Transform::default()))
			.id();
		app.world_mut().spawn((
			AnchorFixPoints::from([(AnchorFixPointKey::new::<()>(11), fix_point)]),
			entity,
		));
		let agent = app
			.world_mut()
			.spawn((
				Anchor::<_NotIgnored>::to(entity).on_fix_point(AnchorFixPointKey::new::<()>(11)),
				Transform::from_scale(Vec3::new(3., 4., 5.)),
			))
			.id();

		_ = app
			.world_mut()
			.run_system_once(Anchor::<_NotIgnored>::system)?;

		assert_eq!(
			Some(&Transform::from_scale(Vec3::new(3., 4., 5.))),
			app.world().entity(agent).get::<Transform>()
		);
		Ok(())
	}

	#[test]
	fn apply_filter() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = PersistentEntity::default();
		let fix_point = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(4., 11., 9.))
			.id();
		app.world_mut().spawn((
			AnchorFixPoints::from([(AnchorFixPointKey::new::<()>(11), fix_point)]),
			entity,
		));
		let agent = app
			.world_mut()
			.spawn((
				Anchor::<_NotIgnored>::to(entity).on_fix_point(AnchorFixPointKey::new::<()>(11)),
				Transform::default(),
				_Ignore,
			))
			.id();

		_ = app
			.world_mut()
			.run_system_once(Anchor::<_NotIgnored>::system)?;

		assert_eq!(
			Some(&Transform::default()),
			app.world().entity(agent).get::<Transform>()
		);
		Ok(())
	}

	#[test]
	fn fix_point_missing() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = PersistentEntity::default();
		app.world_mut().spawn(entity);
		_ = app
			.world_mut()
			.spawn((
				Anchor::<_NotIgnored>::to(entity).on_fix_point(AnchorFixPointKey::new::<()>(11)),
				Transform::default(),
			))
			.id();

		let errors = app
			.world_mut()
			.run_system_once(Anchor::<_NotIgnored>::system)?;

		assert_eq!(vec![Err(AnchorError::FixPointsMissingOn(entity))], errors);
		Ok(())
	}

	#[test]
	fn fix_point_entity_missing() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = PersistentEntity::default();
		app.world_mut().spawn((AnchorFixPoints::default(), entity));
		app.world_mut().spawn((
			Anchor::<_NotIgnored>::to(entity).on_fix_point(AnchorFixPointKey::new::<()>(11)),
			Transform::default(),
		));

		let errors = app
			.world_mut()
			.run_system_once(Anchor::<_NotIgnored>::system)?;

		assert_eq!(
			vec![Err(AnchorError::NoFixPointEntityFor(
				AnchorFixPointKey::new::<()>(11)
			))],
			errors
		);
		Ok(())
	}

	#[test]
	fn transform_missing_on_fix_point() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = PersistentEntity::default();
		let fix_point = app.world_mut().spawn_empty().id();
		app.world_mut().spawn((
			AnchorFixPoints::from([(AnchorFixPointKey::new::<()>(11), fix_point)]),
			entity,
		));
		_ = app
			.world_mut()
			.spawn((
				Anchor::<_NotIgnored>::to(entity).on_fix_point(AnchorFixPointKey::new::<()>(11)),
				Transform::default(),
			))
			.id();

		let errors = app
			.world_mut()
			.run_system_once(Anchor::<_NotIgnored>::system)?;

		assert_eq!(
			vec![Err(AnchorError::GlobalTransformMissingOn(fix_point))],
			errors
		);
		Ok(())
	}

	#[test]
	fn track() {
		let mut anchor_points = AnchorFixPoints::default();

		anchor_points.track(Entity::from_raw(42), &SpawnerFixPoint(Spawner::Center));

		assert_eq!(
			AnchorFixPoints::from([(
				AnchorFixPointKey::from(SpawnerFixPoint(Spawner::Center)),
				Entity::from_raw(42)
			)]),
			anchor_points
		);
	}

	#[test]
	fn is_tracking() {
		let anchor_points = AnchorFixPoints::from([(
			AnchorFixPointKey::from(SpawnerFixPoint(Spawner::Center)),
			Entity::from_raw(42),
		)]);

		assert!(anchor_points.is_tracking(&Entity::from_raw(42)));
	}

	#[test]
	fn is_tracking_false_on_type_mismatch() {
		let anchor_points =
			AnchorFixPoints::from([(AnchorFixPointKey::new::<()>(42), Entity::from_raw(42))]);

		assert!(!anchor_points.is_tracking(&Entity::from_raw(42)));
	}

	#[test]
	fn untrack() {
		let mut anchor_points = AnchorFixPoints::from([(
			AnchorFixPointKey::from(SpawnerFixPoint(Spawner::Center)),
			Entity::from_raw(42),
		)]);

		anchor_points.untrack(&Entity::from_raw(42));

		assert_eq!(AnchorFixPoints::default(), anchor_points);
	}

	#[test]
	fn do_not_untrack_on_type_mismatch() {
		let mut anchor_points =
			AnchorFixPoints::from([(AnchorFixPointKey::new::<()>(42), Entity::from_raw(42))]);

		anchor_points.untrack(&Entity::from_raw(42));

		assert_eq!(
			AnchorFixPoints::from([(AnchorFixPointKey::new::<()>(42), Entity::from_raw(42))]),
			anchor_points
		);
	}
}
