use crate::skill::{Spawner, Target};
use bevy::{
	math::Vec3,
	transform::components::{GlobalTransform, Transform},
};

pub(crate) fn look_from_spawner(agent: &mut Transform, spawner: &Spawner, target: &Target) {
	match target.collision_info.clone().and_then(|ci| ci.root) {
		Some(root) => use_target_transform(agent, root.component),
		_ => use_ray(agent, spawner, target),
	}
}

fn use_target_transform(agent: &mut Transform, target: GlobalTransform) {
	let target = target.translation();
	look_horizontally(agent, target);
}

fn use_ray(agent: &mut Transform, spawner: &Spawner, target: &Target) {
	let spawner = spawner.0.translation();
	let ray = target.ray;
	let Some(ray_length) = ray.intersect_plane(spawner, Vec3::Y) else {
		return;
	};
	let target = ray.origin + ray.direction * ray_length;

	look_horizontally(agent, target);
}

fn look_horizontally(agent: &mut Transform, target: Vec3) {
	agent.look_at(Vec3::new(target.x, agent.translation.y, target.z), Vec3::Y);
}

#[cfg(test)]
mod test_tools {
	use super::*;
	use crate::skill::TransformFN;
	use bevy::{
		app::App,
		ecs::system::Query,
		math::Vec3,
		prelude::Entity,
		transform::components::GlobalTransform,
	};

	pub(crate) fn as_system(
		transform_fn: TransformFN,
		select_info: Target,
	) -> impl Fn(Query<&mut Transform>, Query<&GlobalTransform>) {
		move |mut transforms, global_transforms| {
			let mut transform = transforms.single_mut();
			let global_transform = global_transforms.single();
			transform_fn(&mut transform, &Spawner(*global_transform), &select_info);
		}
	}

	pub fn setup_app(agent: Vec3, spawner: Vec3) -> (App, Entity) {
		let mut app = App::new();
		let agent = app.world.spawn(Transform::from_translation(agent)).id();
		app.world.spawn(GlobalTransform::from_translation(spawner));

		(app, agent)
	}
}

#[cfg(test)]
mod test_look_from_spawner {
	use crate::skill::Outdated;

	use super::{
		test_tools::{as_system, setup_app},
		*,
	};
	use bevy::{
		ecs::entity::Entity,
		math::{Ray, Vec3},
		prelude::Update,
		transform::components::GlobalTransform,
		utils::default,
	};
	use common::{assert_eq_approx, resources::ColliderInfo};

	#[test]
	fn use_odd_ray_and_skill_spawn_for_look_direction() {
		let (mut app, agent) = setup_app(Vec3::new(0., 3., 0.), Vec3::new(0., 3., 0.));
		let target = Target {
			ray: Ray {
				origin: Vec3::new(0., 6., 0.),
				direction: Vec3::new(4., -3., 0.),
			},
			..default()
		};

		app.add_systems(Update, as_system(look_from_spawner, target));
		app.update();

		let agent = app.world.entity(agent);
		let agent = agent.get::<Transform>().unwrap();

		assert_eq_approx!(Vec3::new(1., 0., 0.), agent.forward(), 0.000001);
	}

	#[test]
	fn use_odd_ray_look_direction() {
		let (mut app, agent) = setup_app(Vec3::default(), Vec3::ZERO);
		let target = Target {
			ray: Ray {
				origin: Vec3::new(0., 3., 0.),
				direction: Vec3::new(4., -3., 0.),
			},
			..default()
		};

		app.add_systems(Update, as_system(look_from_spawner, target));
		app.update();

		let agent = app.world.entity(agent);
		let agent = agent.get::<Transform>().unwrap();

		assert_eq_approx!(Vec3::new(1., 0., 0.), agent.forward(), 0.000001);
	}

	#[test]
	fn use_ray_look_direction() {
		let (mut app, agent) = setup_app(Vec3::default(), Vec3::ZERO);
		let target = Target {
			ray: Ray {
				origin: Vec3::new(1., 10., 5.),
				direction: Vec3::NEG_Y,
			},
			..default()
		};

		app.add_systems(Update, as_system(look_from_spawner, target));
		app.update();

		let agent = app.world.entity(agent);
		let agent = agent.get::<Transform>().unwrap();

		assert_eq_approx!(Vec3::new(1., 0., 5.).normalize(), agent.forward(), 0.000001);
	}

	#[test]
	fn look_horizontally() {
		let (mut app, agent) = setup_app(Vec3::new(0., 0., 0.), Vec3::new(0., 3., 0.));
		let target = Target {
			ray: Ray {
				origin: Vec3::new(0., 6., 0.),
				direction: Vec3::new(4., -3., 0.),
			},
			..default()
		};

		app.add_systems(Update, as_system(look_from_spawner, target));
		app.update();

		let agent = app.world.entity(agent);
		let agent = agent.get::<Transform>().unwrap();

		assert_eq_approx!(Vec3::new(1., 0., 0.), agent.forward(), 0.000001);
	}

	#[test]
	fn look_at_target_root_transform() {
		let (mut app, agent) = setup_app(Vec3::new(0., 0., 0.), Vec3::new(0., 3., 0.));
		let target = Target {
			ray: Ray::default(),
			collision_info: Some(ColliderInfo {
				collider: Outdated {
					entity: Entity::from_raw(100),
					component: GlobalTransform::from_xyz(0., 0., 0.),
				},
				root: Some(Outdated {
					entity: Entity::from_raw(101),
					component: GlobalTransform::from_xyz(10., 0., 20.),
				}),
			}),
		};

		app.add_systems(Update, as_system(look_from_spawner, target));
		app.update();

		let agent = app.world.entity(agent);
		let agent = agent.get::<Transform>().unwrap();

		assert_eq_approx!(
			Vec3::new(10., 0., 20.).normalize(),
			agent.forward(),
			0.000001
		);
	}

	#[test]
	fn look_at_target_root_transform_horizontally() {
		let (mut app, agent) = setup_app(Vec3::new(0., 10., 0.), Vec3::new(0., 3., 0.));
		let target = Target {
			ray: Ray::default(),
			collision_info: Some(ColliderInfo {
				collider: Outdated {
					entity: Entity::from_raw(100),
					component: GlobalTransform::from_xyz(0., 0., 0.),
				},
				root: Some(Outdated {
					entity: Entity::from_raw(101),
					component: GlobalTransform::from_xyz(10., 0., 20.),
				}),
			}),
		};

		app.add_systems(Update, as_system(look_from_spawner, target));
		app.update();

		let agent = app.world.entity(agent);
		let agent = agent.get::<Transform>().unwrap();

		assert_eq_approx!(
			Vec3::new(10., 0., 20.).normalize(),
			agent.forward(),
			0.000001
		);
	}
}
