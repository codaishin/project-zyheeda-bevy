use crate::components::GroundTarget;
use bevy::{
	ecs::{
		bundle::Bundle,
		entity::Entity,
		query::Added,
		system::{Commands, Query},
	},
	math::{
		primitives::{Direction3d, Plane3d},
		Vec3,
	},
	transform::components::Transform,
};
use common::traits::{from_transform::FromTransform, try_insert_on::TryInsertOn};
use std::ops::Deref;

pub(crate) fn ground_target<TBundle: Bundle + FromTransform>(
	mut commands: Commands,
	ground_targets: Query<(Entity, &GroundTarget), Added<GroundTarget>>,
	transforms: Query<&Transform>,
) {
	for (id, ground_target) in &ground_targets {
		let Some((c_translation, c_forward)) = caster_data(&transforms, ground_target) else {
			continue;
		};

		let mut target_translation = match intersect_ground_plane(ground_target) {
			Some(toi) => ground_target.target_ray.origin + ground_target.target_ray.direction * toi,
			None => c_translation,
		};

		correct_for_max_range(&mut target_translation, c_translation, ground_target);

		let transform =
			Transform::from_translation(target_translation).looking_to(*c_forward, Vec3::Y);

		commands.try_insert_on(id, TBundle::from_transform(transform));
	}
}

fn caster_data(
	transforms: &Query<&Transform>,
	ground_target: &GroundTarget,
) -> Option<(Vec3, Direction3d)> {
	let caster_transform = transforms.get(ground_target.caster).ok()?;
	let translation = caster_transform.translation;

	Some((
		Vec3::new(translation.x, 0., translation.z),
		caster_transform.forward(),
	))
}

fn intersect_ground_plane(ground_target: &GroundTarget) -> Option<f32> {
	ground_target
		.target_ray
		.intersect_plane(Vec3::ZERO, Plane3d::new(Vec3::Y))
}

fn correct_for_max_range(
	target_translation: &mut Vec3,
	caster_translation: Vec3,
	ground_target: &GroundTarget,
) {
	let target_direction = *target_translation - caster_translation;
	let max_range = *ground_target.max_range.deref();

	if target_direction.length() <= max_range {
		return;
	}

	*target_translation = caster_translation + target_direction.normalize() * max_range;
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::component::Component,
		math::{primitives::Direction3d, Ray3d, Vec3},
	};
	use common::{
		test_tools::utils::SingleThreadedApp,
		tools::Units,
		traits::clamp_zero_positive::ClampZeroPositive,
	};

	#[derive(Component, Debug, PartialEq)]
	struct _TranslationBundle(Transform);

	impl _TranslationBundle {
		fn from_xyz(x: f32, y: f32, z: f32) -> Self {
			Self(Transform::from_xyz(x, y, z))
		}
	}

	impl FromTransform for _TranslationBundle {
		fn from_transform(transform: Transform) -> Self {
			Self(transform)
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, ground_target::<_TranslationBundle>);

		app
	}

	#[test]
	fn set_transform_at_ray_intersecting_zero_elevation_plane() {
		let mut app = setup();

		let caster = app.world.spawn(Transform::from_xyz(10., 11., 12.)).id();
		let ground_target = app
			.world
			.spawn(GroundTarget {
				caster,
				target_ray: Ray3d::new(Vec3::new(0., 5., 0.), Vec3::new(3., -5., 0.)),
				max_range: Units::new(42.),
			})
			.id();

		app.update();

		let ground_target = app.world.entity(ground_target);

		assert_eq!(
			Some(&_TranslationBundle::from_xyz(3., 0., 0.)),
			ground_target.get::<_TranslationBundle>(),
		)
	}

	#[test]
	fn set_transform_to_caster_zero_elevation_when_ray_not_hitting_zero_elevation_plane() {
		let mut app = setup();

		let caster = app.world.spawn(Transform::from_xyz(10., 11., 12.)).id();
		let ground_target = app
			.world
			.spawn(GroundTarget {
				caster,
				target_ray: Ray3d::new(Vec3::new(0., 5., 0.), Vec3::Y),
				max_range: Units::new(42.),
			})
			.id();

		app.update();

		let ground_target = app.world.entity(ground_target);

		assert_eq!(
			Some(&_TranslationBundle::from_xyz(10., 0., 12.)),
			ground_target.get::<_TranslationBundle>(),
		)
	}

	#[test]
	fn limit_translation_to_be_within_max_range_from_caster() {
		let mut app = setup();

		let caster = app.world.spawn(Transform::from_xyz(2., 0., 0.)).id();
		let ground_target = app
			.world
			.spawn(GroundTarget {
				caster,
				target_ray: Ray3d::new(Vec3::new(10., 3., 0.), Vec3::NEG_Y),
				max_range: Units::new(1.),
			})
			.id();

		app.update();

		let ground_target = app.world.entity(ground_target);

		assert_eq!(
			Some(&_TranslationBundle::from_xyz(3., 0., 0.)),
			ground_target.get::<_TranslationBundle>(),
		)
	}

	#[test]
	fn look_towards_caster_forward() {
		let mut app = setup();

		let caster = app
			.world
			.spawn(Transform::default().looking_to(Vec3::ONE, Vec3::Y))
			.id();
		let ground_target = app
			.world
			.spawn(GroundTarget {
				caster,
				target_ray: Ray3d::new(Vec3::new(10., 3., 0.), Vec3::NEG_Y),
				max_range: Units::new(42.),
			})
			.id();

		app.update();

		let ground_target = app.world.entity(ground_target);

		assert_eq!(
			Some(&_TranslationBundle::from_transform(
				Transform::from_xyz(10., 0., 0.).looking_to(Vec3::ONE, Vec3::Y)
			)),
			ground_target.get::<_TranslationBundle>(),
		)
	}

	#[test]
	fn only_set_transform_when_added() {
		let mut app = setup();

		let caster = app.world.spawn(Transform::from_xyz(10., 11., 12.)).id();
		let ground_target = app
			.world
			.spawn(GroundTarget {
				caster,
				target_ray: Ray3d::new(Vec3::new(0., 5., 0.), Vec3::new(3., -5., 0.)),
				max_range: Units::new(42.),
			})
			.id();

		app.update();

		app.world
			.entity_mut(ground_target)
			.get_mut::<GroundTarget>()
			.unwrap()
			.target_ray
			.direction = Direction3d::NEG_Y;

		app.update();

		let ground_target = app.world.entity(ground_target);

		assert_eq!(
			Some(&_TranslationBundle::from_xyz(3., 0., 0.)),
			ground_target.get::<_TranslationBundle>(),
		)
	}
}
