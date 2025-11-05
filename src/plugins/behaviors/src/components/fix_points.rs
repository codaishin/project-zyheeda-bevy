pub(crate) mod fix_point;

use super::{Always, Once};
use crate::{
	components::fix_points::fix_point::{FixPointOf, FixPointSpawner},
	traits::has_filter::HasFilter,
};
use bevy::{ecs::entity::EntityHashSet, prelude::*};
use common::{
	components::persistent_entity::PersistentEntity,
	errors::{ErrorData, Level},
	traits::{accessors::get::Get, handles_skill_behaviors::SkillSpawner, or_ok::OrOk},
	zyheeda_commands::ZyheedaCommands,
};
use std::{any::type_name, collections::HashMap, fmt::Display, marker::PhantomData};

#[derive(Component, Debug, PartialEq, Clone, Copy)]
#[require(Transform)]
pub(crate) struct Anchor<TFilter> {
	pub(crate) target: PersistentEntity,
	pub(crate) skill_spawner: SkillSpawner,
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
	pub(crate) fn to_target<TEntity>(target: TEntity) -> AnchorBuilder<TFilter>
	where
		TEntity: Into<PersistentEntity>,
	{
		AnchorBuilder {
			target: target.into(),
			_p: PhantomData,
		}
	}

	pub(crate) fn with_target_rotation(mut self) -> Self {
		self.use_target_rotation = true;
		self
	}

	pub(crate) fn system(
		commands: ZyheedaCommands,
		mut agents: Query<(&Self, &mut Transform), <Self as HasFilter>::TFilter>,
		fix_points: Query<(&FixPoints, &GlobalTransform)>,
		spawners: Query<&FixPointSpawner>,
		transforms: Query<&GlobalTransform>,
	) -> Result<(), Vec<AnchorError>> {
		agents
			.iter_mut()
			.filter_map(|(anchor, mut anchor_transform)| {
				let target = commands.get(&anchor.target)?;
				let Ok((FixPoints(fix_points), target_transform)) = fix_points.get(target) else {
					return Some(AnchorError::FixPointsMissingOn(anchor.target));
				};
				let Some(fix_point) = fix_points.iter().find(matching(anchor, &spawners)) else {
					return Some(AnchorError::NoFixPointEntityFor(anchor.skill_spawner));
				};
				let Ok(fix_point_transform) = transforms.get(*fix_point) else {
					return Some(AnchorError::GlobalTransformMissingOn(*fix_point));
				};

				anchor_transform.translation = fix_point_transform.translation();
				let rotation = match anchor.use_target_rotation {
					true => target_transform.rotation(),
					false => fix_point_transform.rotation(),
				};
				anchor_transform.rotation = rotation;

				None
			})
			.collect::<Vec<_>>()
			.or_ok(|| ())
	}
}

fn matching<TFilter>(
	anchor: &Anchor<TFilter>,
	spawners: &Query<&FixPointSpawner>,
) -> impl Fn(&&Entity) -> bool {
	move |e| {
		let Ok(FixPointSpawner(spawner)) = spawners.get(**e) else {
			return false;
		};
		spawner == &anchor.skill_spawner
	}
}

pub(crate) struct AnchorBuilder<TFilter> {
	target: PersistentEntity,
	_p: PhantomData<TFilter>,
}

impl<TFilter> AnchorBuilder<TFilter> {
	pub(crate) fn on_spawner(self, spawner: SkillSpawner) -> Anchor<TFilter> {
		Anchor {
			target: self.target,
			skill_spawner: spawner,
			use_target_rotation: false,
			_p: PhantomData,
		}
	}
}

#[derive(Component, Debug, PartialEq, Clone, Default)]
#[relationship_target(relationship = FixPointOf)]
#[require(GlobalTransform)]
pub struct FixPoints(EntityHashSet);

#[derive(Component, Debug, PartialEq, Clone, Default)]
pub struct FixPointsDefinition(pub(crate) HashMap<String, SkillSpawner>);

#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum AnchorError {
	NoFixPointEntityFor(SkillSpawner),
	FixPointsMissingOn(PersistentEntity),
	GlobalTransformMissingOn(Entity),
}

impl Display for AnchorError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			AnchorError::FixPointsMissingOn(entity) => {
				let type_name = type_name::<FixPoints>();
				write!(f, "{entity:?}: {type_name} missing")
			}
			AnchorError::GlobalTransformMissingOn(entity) => {
				let type_name = type_name::<GlobalTransform>();
				write!(f, "{entity}: {type_name} missing")
			}
			AnchorError::NoFixPointEntityFor(entity) => {
				write!(f, "{entity:?} missing")
			}
		}
	}
}

impl ErrorData for AnchorError {
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl Display {
		"Anchor error"
	}

	fn into_details(self) -> impl Display {
		self
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		tools::action_key::slot::SlotKey,
		traits::register_persistent_entities::RegisterPersistentEntities,
	};
	use std::sync::LazyLock;
	use testing::SingleThreadedApp;

	struct _WithoutIgnore;

	#[derive(Component)]
	struct _Ignore;

	impl HasFilter for Anchor<_WithoutIgnore> {
		type TFilter = Without<_Ignore>;
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.register_persistent_entities();

		app
	}

	static AGENT: LazyLock<PersistentEntity> = LazyLock::new(PersistentEntity::default);

	#[test]
	fn copy_location_translation() -> Result<(), RunSystemError> {
		let mut app = setup();
		let spawner_key = SkillSpawner::Slot(SlotKey(22));
		let agent = app.world_mut().spawn(*AGENT).id();
		app.world_mut().spawn((
			FixPointOf(agent),
			FixPointSpawner(spawner_key),
			GlobalTransform::from_xyz(4., 11., 9.),
		));
		let anchor = app
			.world_mut()
			.spawn(Anchor::<_WithoutIgnore>::to_target(*AGENT).on_spawner(spawner_key))
			.id();

		_ = app
			.world_mut()
			.run_system_once(Anchor::<_WithoutIgnore>::system)?;

		assert_eq!(
			Some(&Transform::from_xyz(4., 11., 9.)),
			app.world().entity(anchor).get::<Transform>()
		);
		Ok(())
	}

	#[test]
	fn copy_location_rotation_of_fix_point() -> Result<(), RunSystemError> {
		let mut app = setup();
		let spawner_key = SkillSpawner::Slot(SlotKey(22));
		let agent = app.world_mut().spawn(*AGENT).id();
		app.world_mut().spawn((
			FixPointOf(agent),
			FixPointSpawner(spawner_key),
			GlobalTransform::from(
				Transform::from_xyz(4., 11., 9.).looking_to(Dir3::NEG_Z, Dir3::Y),
			),
		));
		let anchor = app
			.world_mut()
			.spawn(Anchor::<_WithoutIgnore>::to_target(*AGENT).on_spawner(spawner_key))
			.id();

		_ = app
			.world_mut()
			.run_system_once(Anchor::<_WithoutIgnore>::system)?;

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
		app.world_mut().spawn((
			FixPointOf(agent),
			FixPointSpawner(spawner_key),
			GlobalTransform::from_xyz(4., 11., 9.),
		));
		let anchor = app
			.world_mut()
			.spawn(
				Anchor::<_WithoutIgnore>::to_target(*AGENT)
					.on_spawner(spawner_key)
					.with_target_rotation(),
			)
			.id();

		_ = app
			.world_mut()
			.run_system_once(Anchor::<_WithoutIgnore>::system)?;

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
		app.world_mut().spawn((
			FixPointOf(agent),
			FixPointSpawner(spawner_key),
			GlobalTransform::from(Transform::from_xyz(4., 11., 9.).with_scale(Vec3::splat(2.))),
		));
		let anchor = app
			.world_mut()
			.spawn(Anchor::<_WithoutIgnore>::to_target(*AGENT).on_spawner(spawner_key))
			.id();

		_ = app
			.world_mut()
			.run_system_once(Anchor::<_WithoutIgnore>::system)?;

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
		let agent = app.world_mut().spawn(*AGENT).id();
		app.world_mut().spawn((
			FixPointOf(agent),
			FixPointSpawner(spawner_key),
			GlobalTransform::from_xyz(4., 11., 9.),
		));
		let anchor = app
			.world_mut()
			.spawn((
				Anchor::<_WithoutIgnore>::to_target(*AGENT).on_spawner(spawner_key),
				_Ignore,
			))
			.id();

		_ = app
			.world_mut()
			.run_system_once(Anchor::<_WithoutIgnore>::system)?;

		assert_eq!(
			Some(&Transform::default()),
			app.world().entity(anchor).get::<Transform>()
		);
		Ok(())
	}

	#[test]
	fn fix_point_missing() -> Result<(), RunSystemError> {
		let mut app = setup();
		let spawner_key = SkillSpawner::Slot(SlotKey(22));
		app.world_mut().spawn(*AGENT);
		app.world_mut().spawn((
			FixPointSpawner(spawner_key),
			GlobalTransform::from_xyz(4., 11., 9.),
		));
		app.world_mut()
			.spawn(Anchor::<_WithoutIgnore>::to_target(*AGENT).on_spawner(spawner_key));

		let errors = app
			.world_mut()
			.run_system_once(Anchor::<_WithoutIgnore>::system)?;

		assert_eq!(Err(vec![AnchorError::FixPointsMissingOn(*AGENT)]), errors);
		Ok(())
	}

	#[test]
	fn fix_point_entity_missing() -> Result<(), RunSystemError> {
		let mut app = setup();
		let spawner_key = SkillSpawner::Slot(SlotKey(22));
		let agent = app.world_mut().spawn(*AGENT).id();
		app.world_mut().spawn((
			FixPointOf(agent),
			FixPointSpawner(SkillSpawner::Slot(SlotKey(11))),
			GlobalTransform::from_xyz(4., 11., 9.),
		));
		app.world_mut()
			.spawn(Anchor::<_WithoutIgnore>::to_target(*AGENT).on_spawner(spawner_key));

		let errors = app
			.world_mut()
			.run_system_once(Anchor::<_WithoutIgnore>::system)?;

		assert_eq!(
			Err(vec![AnchorError::NoFixPointEntityFor(spawner_key)]),
			errors
		);
		Ok(())
	}

	#[test]
	fn transform_missing_on_fix_point() -> Result<(), RunSystemError> {
		let mut app = setup();
		let spawner_key = SkillSpawner::Slot(SlotKey(22));
		let agent = app.world_mut().spawn(*AGENT).id();
		let spawner = app
			.world_mut()
			.spawn((FixPointOf(agent), FixPointSpawner(spawner_key)))
			.id();
		app.world_mut()
			.spawn(Anchor::<_WithoutIgnore>::to_target(*AGENT).on_spawner(spawner_key));

		let errors = app
			.world_mut()
			.run_system_once(Anchor::<_WithoutIgnore>::system)?;

		assert_eq!(
			Err(vec![AnchorError::GlobalTransformMissingOn(spawner)]),
			errors
		);
		Ok(())
	}
}
