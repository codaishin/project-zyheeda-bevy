pub(crate) mod fix_point;

use super::{Always, Once};
use crate::{components::fix_points::fix_point::FixPoint, traits::has_filter::HasFilter};
use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	errors::{Error, Level},
	tools::Index,
	traits::{
		accessors::get::GetMut,
		or_ok::OrOk,
		track::{IsTracking, Track, Untrack},
	},
	zyheeda_commands::ZyheedaCommands,
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
	pub(crate) use_target_rotation: bool,
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
	pub(crate) fn to_target(target: PersistentEntity) -> AnchorBuilder<TFilter> {
		AnchorBuilder {
			target,
			_p: PhantomData,
		}
	}

	pub(crate) fn with_target_rotation(mut self) -> Self {
		self.use_target_rotation = true;
		self
	}

	pub(crate) fn system(
		mut commands: ZyheedaCommands,
		mut agents: Query<(&Self, &mut Transform), <Self as HasFilter>::TFilter>,
		fix_points: Query<(&FixPoints, &GlobalTransform)>,
		transforms: Query<&GlobalTransform>,
	) -> Result<(), Vec<AnchorError>> {
		agents
			.iter_mut()
			.filter_map(|(anchor, mut anchor_transform)| {
				let target = commands.get_mut(&anchor.target)?.id();
				let Ok((FixPoints(fix_points), transform)) = fix_points.get(target) else {
					return Some(AnchorError::FixPointsMissingOn(anchor.target));
				};
				let Some(fix_point) = fix_points.get(&anchor.fix_point_key).copied() else {
					return Some(AnchorError::NoFixPointEntityFor(anchor.fix_point_key));
				};
				let Ok(fix_point) = transforms.get(fix_point) else {
					return Some(AnchorError::GlobalTransformMissingOn(fix_point));
				};

				anchor_transform.translation = fix_point.translation();
				let rotation = match anchor.use_target_rotation {
					true => transform.rotation(),
					false => fix_point.rotation(),
				};
				anchor_transform.rotation = rotation;

				None
			})
			.collect::<Vec<_>>()
			.or_ok(|| ())
	}
}

pub(crate) struct AnchorBuilder<TFilter> {
	target: PersistentEntity,
	_p: PhantomData<TFilter>,
}

impl<TFilter> AnchorBuilder<TFilter> {
	/// Anchor to selected fix point with that fix point's rotation
	pub(crate) fn on_fix_point<TFixPoint>(self, fix_point_key: TFixPoint) -> Anchor<TFilter>
	where
		TFixPoint: Into<AnchorFixPointKey>,
	{
		Anchor {
			target: self.target,
			fix_point_key: fix_point_key.into(),
			use_target_rotation: false,
			_p: PhantomData,
		}
	}
}

#[derive(Component, Debug, PartialEq, Clone, Default)]
#[require(GlobalTransform)]
pub struct FixPoints(HashMap<AnchorFixPointKey, Entity>);

impl<T> Track<FixPoint<T>> for FixPoints
where
	T: 'static + Copy + Into<Index<usize>>,
{
	fn track(&mut self, entity: Entity, spawner_fix_point: &FixPoint<T>) {
		self.0.insert((*spawner_fix_point).into(), entity);
	}
}

impl<T> Untrack<FixPoint<T>> for FixPoints
where
	T: 'static,
{
	fn untrack(&mut self, entity: &Entity) {
		self.0
			.retain(|k, e| e != entity || k.source_type != TypeId::of::<FixPoint<T>>());
	}
}

impl<T> IsTracking<FixPoint<T>> for FixPoints
where
	T: 'static,
{
	fn is_tracking(&self, entity: &Entity) -> bool {
		self.0
			.iter()
			.any(|(k, e)| e == entity && k.source_type == TypeId::of::<FixPoint<T>>())
	}
}

impl<const N: usize, TKey> From<[(TKey, Entity); N]> for FixPoints
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
				let type_name = type_name::<FixPoints>();
				Self::Single {
					msg: format!("{entity:?}: {type_name} missing"),
					lvl: Level::Error,
				}
			}
			AnchorError::GlobalTransformMissingOn(entity) => {
				let type_name = type_name::<GlobalTransform>();
				Self::Single {
					msg: format!("{entity}: {type_name} missing"),
					lvl: Level::Error,
				}
			}
			AnchorError::NoFixPointEntityFor(anchor_fix_point_key) => Self::Single {
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
		handles_skill_behaviors::SkillSpawner,
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
			FixPoints::from([(AnchorFixPointKey::new::<()>(11), fix_point)]),
			entity,
		));
		let agent = app
			.world_mut()
			.spawn((
				Anchor::<_NotIgnored>::to_target(entity)
					.on_fix_point(AnchorFixPointKey::new::<()>(11)),
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
	fn copy_location_rotation_of_fix_point() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = PersistentEntity::default();
		let fix_point = app
			.world_mut()
			.spawn(GlobalTransform::from(
				Transform::default().looking_at(Vec3::new(0., 0., 1.), Vec3::Y),
			))
			.id();
		app.world_mut().spawn((
			FixPoints::from([(AnchorFixPointKey::new::<()>(11), fix_point)]),
			entity,
		));
		let agent = app
			.world_mut()
			.spawn((
				Anchor::<_NotIgnored>::to_target(entity)
					.on_fix_point(AnchorFixPointKey::new::<()>(11)),
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
	fn copy_rotation_of_target() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = PersistentEntity::default();
		let fix_point = app
			.world_mut()
			.spawn(GlobalTransform::from(
				Transform::default().looking_at(Vec3::new(1., 0., 0.), Vec3::Y),
			))
			.id();
		app.world_mut().spawn((
			FixPoints::from([(AnchorFixPointKey::new::<()>(11), fix_point)]),
			entity,
			GlobalTransform::from(Transform::default().looking_at(Vec3::new(0., 0., 1.), Vec3::Y)),
		));
		let agent = app
			.world_mut()
			.spawn((
				Anchor::<_NotIgnored>::to_target(entity)
					.on_fix_point(AnchorFixPointKey::new::<()>(11))
					.with_target_rotation(),
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
			FixPoints::from([(AnchorFixPointKey::new::<()>(11), fix_point)]),
			entity,
		));
		let agent = app
			.world_mut()
			.spawn((
				Anchor::<_NotIgnored>::to_target(entity)
					.on_fix_point(AnchorFixPointKey::new::<()>(11)),
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
			FixPoints::from([(AnchorFixPointKey::new::<()>(11), fix_point)]),
			entity,
		));
		let agent = app
			.world_mut()
			.spawn((
				Anchor::<_NotIgnored>::to_target(entity)
					.on_fix_point(AnchorFixPointKey::new::<()>(11)),
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
				Anchor::<_NotIgnored>::to_target(entity)
					.on_fix_point(AnchorFixPointKey::new::<()>(11)),
				Transform::default(),
			))
			.id();

		let errors = app
			.world_mut()
			.run_system_once(Anchor::<_NotIgnored>::system)?;

		assert_eq!(Err(vec![AnchorError::FixPointsMissingOn(entity)]), errors);
		Ok(())
	}

	#[test]
	fn fix_point_entity_missing() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = PersistentEntity::default();
		app.world_mut().spawn((FixPoints::default(), entity));
		app.world_mut().spawn((
			Anchor::<_NotIgnored>::to_target(entity).on_fix_point(AnchorFixPointKey::new::<()>(11)),
			Transform::default(),
		));

		let errors = app
			.world_mut()
			.run_system_once(Anchor::<_NotIgnored>::system)?;

		assert_eq!(
			Err(vec![AnchorError::NoFixPointEntityFor(
				AnchorFixPointKey::new::<()>(11)
			)]),
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
			FixPoints::from([(AnchorFixPointKey::new::<()>(11), fix_point)]),
			entity,
		));
		_ = app
			.world_mut()
			.spawn((
				Anchor::<_NotIgnored>::to_target(entity)
					.on_fix_point(AnchorFixPointKey::new::<()>(11)),
				Transform::default(),
			))
			.id();

		let errors = app
			.world_mut()
			.run_system_once(Anchor::<_NotIgnored>::system)?;

		assert_eq!(
			Err(vec![AnchorError::GlobalTransformMissingOn(fix_point)]),
			errors
		);
		Ok(())
	}

	#[test]
	fn track() {
		let mut anchor_points = FixPoints::default();

		anchor_points.track(Entity::from_raw(42), &FixPoint(SkillSpawner::Center));

		assert_eq!(
			FixPoints::from([(
				AnchorFixPointKey::from(FixPoint(SkillSpawner::Center)),
				Entity::from_raw(42)
			)]),
			anchor_points
		);
	}

	#[test]
	fn is_tracking() {
		let anchor_points = FixPoints::from([(
			AnchorFixPointKey::from(FixPoint(SkillSpawner::Center)),
			Entity::from_raw(42),
		)]);

		assert!(IsTracking::<FixPoint<SkillSpawner>>::is_tracking(
			&anchor_points,
			&Entity::from_raw(42)
		));
	}

	#[test]
	fn is_tracking_false_on_type_mismatch() {
		let anchor_points =
			FixPoints::from([(AnchorFixPointKey::new::<()>(42), Entity::from_raw(42))]);

		assert!(!IsTracking::<FixPoint<SkillSpawner>>::is_tracking(
			&anchor_points,
			&Entity::from_raw(42)
		));
	}

	#[test]
	fn untrack() {
		let mut anchor_points = FixPoints::from([(
			AnchorFixPointKey::from(FixPoint(SkillSpawner::Center)),
			Entity::from_raw(42),
		)]);

		Untrack::<FixPoint<SkillSpawner>>::untrack(&mut anchor_points, &Entity::from_raw(42));

		assert_eq!(FixPoints::default(), anchor_points);
	}

	#[test]
	fn do_not_untrack_on_type_mismatch() {
		let mut anchor_points =
			FixPoints::from([(AnchorFixPointKey::new::<()>(42), Entity::from_raw(42))]);

		Untrack::<FixPoint<SkillSpawner>>::untrack(&mut anchor_points, &Entity::from_raw(42));

		assert_eq!(
			FixPoints::from([(AnchorFixPointKey::new::<()>(42), Entity::from_raw(42))]),
			anchor_points
		);
	}
}
