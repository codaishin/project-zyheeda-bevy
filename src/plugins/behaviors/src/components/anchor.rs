pub(crate) mod spawner_fix_point;

use super::{Always, Once};
use crate::{
	components::anchor::spawner_fix_point::SpawnerFixPoint,
	traits::has_filter::HasFilter,
};
use bevy::prelude::*;
use common::{
	errors::{Error, Level},
	tools::action_key::slot::{Side, SlotKey},
	traits::{
		handles_skill_behaviors::Spawner,
		track::{IsTracking, Track, Untrack},
	},
};
use std::{any::type_name, collections::HashMap, marker::PhantomData};

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub(crate) struct Anchor<TFilter> {
	pub(crate) target: Entity,
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
	pub(crate) fn to(target: Entity) -> AnchorBuilder<TFilter> {
		AnchorBuilder {
			target,
			_p: PhantomData,
		}
	}

	pub(crate) fn system(
		mut agents: Query<(&Self, &mut Transform), <Self as HasFilter>::TFilter>,
		fix_points: Query<&AnchorFixPoints>,
		transforms: Query<&GlobalTransform>,
	) -> Vec<Result<(), AnchorError>> {
		agents
			.iter_mut()
			.filter_map(|(anchor, mut anchor_transform)| {
				let Ok(AnchorFixPoints(fix_points)) = fix_points.get(anchor.target) else {
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
	target: Entity,
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
	fn track(&mut self, entity: Entity, SpawnerFixPoint(spawner): &SpawnerFixPoint) {
		self.0.insert((*spawner).into(), entity);
	}
}

impl Untrack<SpawnerFixPoint> for AnchorFixPoints {
	fn untrack(&mut self, entity: &Entity) {
		self.0.retain(|_, e| e == entity);
	}
}

impl IsTracking<SpawnerFixPoint> for AnchorFixPoints {
	fn is_tracking(&self, entity: &Entity) -> bool {
		self.0.values().any(|e| e == entity)
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
pub struct AnchorFixPointKey(pub(crate) usize);

impl From<Spawner> for AnchorFixPointKey {
	fn from(value: Spawner) -> Self {
		match value {
			Spawner::Center => AnchorFixPointKey(0),
			Spawner::Slot(SlotKey::BottomHand(Side::Left)) => AnchorFixPointKey(1),
			Spawner::Slot(SlotKey::BottomHand(Side::Right)) => AnchorFixPointKey(2),
			Spawner::Slot(SlotKey::TopHand(Side::Left)) => AnchorFixPointKey(3),
			Spawner::Slot(SlotKey::TopHand(Side::Right)) => AnchorFixPointKey(4),
		}
	}
}

#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum AnchorError {
	NoFixPointEntityFor(AnchorFixPointKey),
	FixPointsMissingOn(Entity),
	GlobalTransformMissingOn(Entity),
}

impl From<AnchorError> for Error {
	fn from(error: AnchorError) -> Self {
		match error {
			AnchorError::FixPointsMissingOn(entity) => {
				let type_name = type_name::<AnchorFixPoints>();
				Self {
					msg: format!("{entity}: {type_name} missing"),
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
	use common::{test_tools::utils::SingleThreadedApp, traits::iteration::IterFinite};
	use std::collections::HashSet;

	struct _NotIgnored;

	#[derive(Component)]
	struct _Ignore;

	impl HasFilter for Anchor<_NotIgnored> {
		type TFilter = Without<_Ignore>;
	}

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn copy_location_translation() -> Result<(), RunSystemError> {
		let mut app = setup();
		let fix_point = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(4., 11., 9.))
			.id();
		let entity = app
			.world_mut()
			.spawn(AnchorFixPoints::from([(AnchorFixPointKey(11), fix_point)]))
			.id();
		let agent = app
			.world_mut()
			.spawn((
				Anchor::<_NotIgnored>::to(entity).on_fix_point(AnchorFixPointKey(11)),
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
		let fix_point = app
			.world_mut()
			.spawn(GlobalTransform::from(
				Transform::default().looking_at(Vec3::new(0., 0., 1.), Vec3::Y),
			))
			.id();
		let entity = app
			.world_mut()
			.spawn(AnchorFixPoints::from([(AnchorFixPointKey(11), fix_point)]))
			.id();
		let agent = app
			.world_mut()
			.spawn((
				Anchor::<_NotIgnored>::to(entity).on_fix_point(AnchorFixPointKey(11)),
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
		let fix_point = app
			.world_mut()
			.spawn(GlobalTransform::from(Transform::default()))
			.id();
		let entity = app
			.world_mut()
			.spawn(AnchorFixPoints::from([(AnchorFixPointKey(11), fix_point)]))
			.id();
		let agent = app
			.world_mut()
			.spawn((
				Anchor::<_NotIgnored>::to(entity).on_fix_point(AnchorFixPointKey(11)),
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
		let fix_point = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(4., 11., 9.))
			.id();
		let entity = app
			.world_mut()
			.spawn(AnchorFixPoints::from([(AnchorFixPointKey(11), fix_point)]))
			.id();
		let agent = app
			.world_mut()
			.spawn((
				Anchor::<_NotIgnored>::to(entity).on_fix_point(AnchorFixPointKey(11)),
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
		let entity = app.world_mut().spawn_empty().id();
		_ = app
			.world_mut()
			.spawn((
				Anchor::<_NotIgnored>::to(entity).on_fix_point(AnchorFixPointKey(11)),
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
		let entity = app.world_mut().spawn(AnchorFixPoints::default()).id();
		app.world_mut().spawn((
			Anchor::<_NotIgnored>::to(entity).on_fix_point(AnchorFixPointKey(11)),
			Transform::default(),
		));

		let errors = app
			.world_mut()
			.run_system_once(Anchor::<_NotIgnored>::system)?;

		assert_eq!(
			vec![Err(AnchorError::NoFixPointEntityFor(AnchorFixPointKey(11)))],
			errors
		);
		Ok(())
	}

	#[test]
	fn transform_missing_on_fix_point() -> Result<(), RunSystemError> {
		let mut app = setup();
		let fix_point = app.world_mut().spawn_empty().id();
		let entity = app
			.world_mut()
			.spawn(AnchorFixPoints::from([(AnchorFixPointKey(11), fix_point)]))
			.id();
		_ = app
			.world_mut()
			.spawn((
				Anchor::<_NotIgnored>::to(entity).on_fix_point(AnchorFixPointKey(11)),
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
	fn spawner_to_anchor_fix_point_key_has_no_duplicate_values() {
		let slot_keys = SlotKey::iterator()
			.map(Spawner::Slot)
			.chain(std::iter::once(Spawner::Center))
			.collect::<Vec<_>>();

		let anchor_keys = slot_keys
			.iter()
			.copied()
			.map(AnchorFixPointKey::from)
			.collect::<HashSet<_>>();

		assert_eq!(slot_keys.len(), anchor_keys.len());
	}
}
