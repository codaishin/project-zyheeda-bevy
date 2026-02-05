use crate::{
	components::fix_points::{Anchor, AnchorError, FixPoints, fix_point::FixPointSpawner},
	traits::query_filter_definition::QueryFilterDefinition,
};
use bevy::prelude::*;
use common::{
	traits::{accessors::get::Get, or_ok::OrOk},
	zyheeda_commands::ZyheedaCommands,
};

impl<TFilter> Anchor<TFilter>
where
	Self: QueryFilterDefinition + 'static,
{
	pub(crate) fn system(
		commands: ZyheedaCommands,
		mut agents: Query<(&Self, &mut Transform), <Self as QueryFilterDefinition>::TFilter>,
		fix_points: Query<(&FixPoints, &GlobalTransform)>,
		spawners: Query<&FixPointSpawner>,
		transforms: Query<&GlobalTransform>,
	) -> Result<(), Vec<AnchorError>> {
		agents
			.iter_mut()
			.filter_map(|(anchor, mut anchor_transform)| {
				let target = commands.get(&anchor.target)?;
				let Ok((fix_points, target_transform)) = fix_points.get(target) else {
					return Some(AnchorError::FixPointsMissingOn(anchor.target));
				};
				let Some(fix_point) = fix_points.iter().find(matching(anchor, &spawners)) else {
					return Some(AnchorError::NoFixPointEntityFor(anchor.skill_spawner));
				};
				let Ok(fix_point_transform) = transforms.get(*fix_point) else {
					return Some(AnchorError::GlobalTransformMissingOn(*fix_point));
				};

				let fix_point_translation = fix_point_transform.translation();
				if fix_point_translation.is_nan() {
					return Some(AnchorError::FixPointTranslationNaN(*fix_point));
				}

				anchor_transform.translation = fix_point_translation;
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

#[cfg(test)]
mod tests {
	use crate::components::fix_points::fix_point::FixPointOf;

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
	use std::sync::LazyLock;
	use testing::SingleThreadedApp;

	struct _WithoutIgnore;

	#[derive(Component)]
	struct _Ignore;

	impl QueryFilterDefinition for Anchor<_WithoutIgnore> {
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

	#[test]
	fn fix_point_translation_nan() -> Result<(), RunSystemError> {
		let mut app = setup();
		let spawner_key = SkillSpawner::Slot(SlotKey(22));
		let agent = app.world_mut().spawn(*AGENT).id();
		let spawner = app
			.world_mut()
			.spawn((
				FixPointOf(agent),
				FixPointSpawner(spawner_key),
				GlobalTransform::from_translation(Vec3::NAN),
			))
			.id();
		app.world_mut()
			.spawn(Anchor::<_WithoutIgnore>::to_target(*AGENT).on_spawner(spawner_key));

		let errors = app
			.world_mut()
			.run_system_once(Anchor::<_WithoutIgnore>::system)?;

		assert_eq!(
			Err(vec![AnchorError::FixPointTranslationNaN(spawner)]),
			errors
		);
		Ok(())
	}
}
