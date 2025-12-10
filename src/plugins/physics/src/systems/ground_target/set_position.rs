use crate::components::ground_target::GroundTarget;
use bevy::prelude::*;
use common::{
	traits::{
		accessors::get::{Get, TryApplyOn},
		handles_skill_behaviors::SkillTarget,
	},
	zyheeda_commands::ZyheedaCommands,
};

impl GroundTarget {
	pub(crate) fn set_position(
		mut commands: ZyheedaCommands,
		transforms: Query<&Transform>,
		ground_targets: Query<(Entity, &GroundTarget), Added<GroundTarget>>,
	) {
		for (entity, ground_target) in &ground_targets {
			let Some(mut transform) = ground_target.transform(&commands, transforms) else {
				continue;
			};
			let Some(caster) = commands.get(&ground_target.caster.0) else {
				continue;
			};

			if let Ok(caster) = transforms.get(caster) {
				ground_target.correct_for_max_range(&mut transform, caster);
				Self::sync_forward(&mut transform, caster);
			}

			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(transform);
			});
		}
	}

	fn transform(
		&self,
		commands: &ZyheedaCommands,
		transforms: Query<&Transform>,
	) -> Option<Transform> {
		match self.target {
			SkillTarget::Ground(point) => Some(Transform::from_translation(point)),
			SkillTarget::Entity(persistent_entity) => commands
				.get(&persistent_entity)
				.and_then(|e| transforms.get(e).ok())
				.map(|t| Transform::from_translation(t.translation)),
		}
	}

	fn correct_for_max_range(&self, contact: &mut Transform, caster: &Transform) {
		let direction = contact.translation - caster.translation;
		let max_range = *self.max_cast_range;

		if direction.length() <= max_range {
			return;
		}

		contact.translation = caster.translation + direction.normalize() * max_range;
	}

	fn sync_forward(transform: &mut Transform, caster: &Transform) {
		transform.look_to(caster.forward(), Vec3::Y);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		components::persistent_entity::PersistentEntity,
		tools::Units,
		traits::{
			handles_skill_behaviors::SkillCaster,
			register_persistent_entities::RegisterPersistentEntities,
		},
	};
	use testing::{SingleThreadedApp, assert_eq_approx};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.register_persistent_entities();
		app.add_systems(Update, GroundTarget::set_position);

		app
	}

	#[test]
	fn set_to_ground_target() {
		let mut app = setup();
		let caster = SkillCaster::default();
		app.world_mut().spawn((Transform::default(), *caster));
		let entity = app
			.world_mut()
			.spawn(GroundTarget::with_caster(caster).with_target(Vec3::new(1., 2., 3.)))
			.id();

		app.update();

		assert_eq!(
			Some(&Transform::from_xyz(1., 2., 3.)),
			app.world().entity(entity).get::<Transform>(),
		)
	}

	#[test]
	fn set_to_entity_transform() {
		let mut app = setup();
		let caster = SkillCaster::default();
		let target = PersistentEntity::default();
		app.world_mut().spawn((Transform::default(), *caster));
		app.world_mut()
			.spawn((Transform::from_xyz(3., 7., -1.), target));
		let entity = app
			.world_mut()
			.spawn(GroundTarget::with_caster(caster).with_target(target))
			.id();

		app.update();

		assert_eq!(
			Some(&Transform::from_xyz(3., 7., -1.)),
			app.world().entity(entity).get::<Transform>(),
		)
	}

	#[test]
	fn set_to_entity_transform_with_scale_zero() {
		let mut app = setup();
		let caster = SkillCaster::default();
		let target = PersistentEntity::default();
		app.world_mut().spawn((Transform::default(), *caster));
		app.world_mut().spawn((
			Transform::from_xyz(3., 7., -1.).with_scale(Vec3::splat(42.)),
			target,
		));
		let entity = app
			.world_mut()
			.spawn(GroundTarget::with_caster(caster).with_target(target))
			.id();

		app.update();

		assert_eq!(
			Some(&Transform::from_xyz(3., 7., -1.).with_scale(Vec3::splat(1.))),
			app.world().entity(entity).get::<Transform>(),
		)
	}

	#[test]
	fn limit_by_max_range() {
		let mut app = setup();
		let caster = SkillCaster::default();
		app.world_mut().spawn((Transform::default(), *caster));
		let entity = app
			.world_mut()
			.spawn(
				GroundTarget::with_caster(caster)
					.with_target(Vec3::new(6., 0., 8.))
					.with_max_range(Units::from(5.)),
			)
			.id();

		app.update();

		assert_eq!(
			Some(&Transform::from_xyz(3., 0., 4.)),
			app.world().entity(entity).get::<Transform>(),
		)
	}

	#[test]
	fn limit_by_max_range_when_caster_offset_from_zero() {
		let mut app = setup();
		let caster = SkillCaster::default();
		app.world_mut()
			.spawn((Transform::from_xyz(1., 0., 0.), *caster));
		let entity = app
			.world_mut()
			.spawn(
				GroundTarget::with_caster(caster)
					.with_target(Vec3::new(7., 0., 8.))
					.with_max_range(Units::from(5.)),
			)
			.id();

		app.update();

		assert_eq!(
			Some(&Transform::from_xyz(4., 0., 4.)),
			app.world().entity(entity).get::<Transform>(),
		)
	}

	#[test]
	fn do_not_limit_by_max_range_when_caster_has_no_transform() {
		let mut app = setup();
		let caster = SkillCaster::default();
		app.world_mut().spawn(*caster);
		let entity = app
			.world_mut()
			.spawn(
				GroundTarget::with_caster(caster)
					.with_target(Vec3::new(6., 0., 8.))
					.with_max_range(Units::from(5.)),
			)
			.id();

		app.update();

		assert_eq!(
			Some(&Transform::from_xyz(6., 0., 8.)),
			app.world().entity(entity).get::<Transform>(),
		)
	}

	#[test]
	fn set_forward_to_caster_forward() {
		let mut app = setup();
		let caster = SkillCaster::default();
		app.world_mut().spawn((
			Transform::default().looking_to(Vec3::new(3., 0., 4.), Vec3::Y),
			*caster,
		));
		let entity = app
			.world_mut()
			.spawn(GroundTarget::with_caster(caster).with_target(Vec3::new(1., 0., 1.)))
			.id();

		app.update();

		assert_eq_approx!(
			Some(&Transform::from_xyz(1., 0., 1.).looking_to(Vec3::new(3., 0., 4.), Vec3::Y)),
			app.world().entity(entity).get::<Transform>(),
			0.000001
		)
	}

	#[test]
	fn only_set_transform_when_added() {
		let mut app = setup();
		let caster = SkillCaster::default();
		app.world_mut().spawn((Transform::default(), *caster));
		let entity = app
			.world_mut()
			.spawn(GroundTarget::with_caster(caster).with_target(Vec3::new(1., 0., 1.)))
			.id();

		app.update();
		let mut ground_target = app.world_mut().entity_mut(entity);
		let mut transform = ground_target.get_mut::<Transform>().unwrap();
		*transform = Transform::from_xyz(1., 2., 3.);
		app.update();

		assert_eq!(
			Some(&Transform::from_xyz(1., 2., 3.)),
			app.world().entity(entity).get::<Transform>(),
		)
	}
}
