use bevy::prelude::*;
use common::{tools::Units, traits::try_insert_on::TryInsertOn};

#[derive(Component, Debug, PartialEq, Clone)]
pub(crate) struct GroundTarget {
	pub caster: Entity,
	pub target_ray: Ray3d,
	pub max_cast_range: Units,
}

impl GroundTarget {
	#[cfg(test)]
	pub const DEFAULT_TARGET_RAY: Ray3d = Ray3d {
		origin: Vec3::Y,
		direction: Dir3::NEG_Y,
	};

	#[cfg(test)]
	fn with_caster(caster: Entity) -> Self {
		use common::traits::clamp_zero_positive::ClampZeroPositive;

		GroundTarget {
			caster,
			target_ray: Self::DEFAULT_TARGET_RAY,
			max_cast_range: Units::new(f32::INFINITY),
		}
	}

	#[cfg(test)]
	fn with_target_ray(mut self, ray: Ray3d) -> Self {
		self.target_ray = ray;
		self
	}

	#[cfg(test)]
	fn with_max_range(mut self, max_range: Units) -> Self {
		self.max_cast_range = max_range;
		self
	}
}

impl GroundTarget {
	fn ground_contact(&self) -> Option<Vec3> {
		let toi = self
			.target_ray
			.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))?;

		Some(self.target_ray.origin + self.target_ray.direction * toi)
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

	pub(crate) fn set_position(
		mut commands: Commands,
		transforms: Query<&Transform>,
		ground_targets: Query<(Entity, &GroundTarget), Added<GroundTarget>>,
	) {
		for (entity, ground_target) in &ground_targets {
			let Some(contact) = ground_target.ground_contact() else {
				continue;
			};
			let mut transform = Transform::from_translation(contact);

			if let Ok(caster) = transforms.get(ground_target.caster) {
				ground_target.correct_for_max_range(&mut transform, caster);
				Self::sync_forward(&mut transform, caster);
			}

			commands.try_insert_on(entity, transform);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		assert_eq_approx,
		test_tools::utils::SingleThreadedApp,
		traits::clamp_zero_positive::ClampZeroPositive,
	};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, GroundTarget::set_position);

		app
	}

	#[test]
	fn set_to_intersection_of_target_ray_and_ground_level() {
		let mut app = setup();
		let caster = app.world_mut().spawn(Transform::default()).id();
		let ray = Ray3d {
			origin: Vec3::new(2., 5., 1.),
			direction: Dir3::new_unchecked(Vec3::new(0., -5., 5.).normalize()),
		};
		let entity = app
			.world_mut()
			.spawn(GroundTarget::with_caster(caster).with_target_ray(ray))
			.id();

		app.update();

		assert_eq!(
			Some(&Transform::from_xyz(2., 0., 6.)),
			app.world().entity(entity).get::<Transform>(),
		)
	}

	#[test]
	fn limit_by_max_range() {
		let mut app = setup();
		let caster = app.world_mut().spawn(Transform::default()).id();
		let ray = Ray3d {
			origin: Vec3::new(6., 1., 8.),
			direction: Dir3::new_unchecked(Vec3::new(0., -1., 0.)),
		};
		let entity = app
			.world_mut()
			.spawn(
				GroundTarget::with_caster(caster)
					.with_target_ray(ray)
					.with_max_range(Units::new(5.)),
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
		let caster = app.world_mut().spawn(Transform::from_xyz(1., 0., 0.)).id();
		let ray = Ray3d {
			origin: Vec3::new(7., 1., 8.),
			direction: Dir3::new_unchecked(Vec3::new(0., -1., 0.)),
		};
		let entity = app
			.world_mut()
			.spawn(
				GroundTarget::with_caster(caster)
					.with_target_ray(ray)
					.with_max_range(Units::new(5.)),
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
		let caster = app.world_mut().spawn_empty().id();
		let ray = Ray3d {
			origin: Vec3::new(6., 1., 8.),
			direction: Dir3::new_unchecked(Vec3::new(0., -1., 0.)),
		};
		let entity = app
			.world_mut()
			.spawn(
				GroundTarget::with_caster(caster)
					.with_target_ray(ray)
					.with_max_range(Units::new(5.)),
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
		let caster = app
			.world_mut()
			.spawn(Transform::default().looking_to(Vec3::new(3., 0., 4.), Vec3::Y))
			.id();
		let ray = Ray3d {
			origin: Vec3::new(1., 1., 1.),
			direction: Dir3::new_unchecked(Vec3::new(0., -1., 0.).normalize()),
		};
		let entity = app
			.world_mut()
			.spawn(GroundTarget::with_caster(caster).with_target_ray(ray))
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
		let caster = app.world_mut().spawn(Transform::default()).id();
		let ray = Ray3d {
			origin: Vec3::new(1., 1., 1.),
			direction: Dir3::new_unchecked(Vec3::new(0., -1., 0.).normalize()),
		};
		let entity = app
			.world_mut()
			.spawn(GroundTarget::with_caster(caster).with_target_ray(ray))
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
