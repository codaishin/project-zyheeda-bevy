use crate::{
	components::ColliderRoot,
	resources::{CamRay, ColliderInfo, MouseHover},
	traits::cast_ray::{CastRay, TimeOfImpact},
};
use bevy::ecs::{
	entity::Entity,
	system::{Commands, Query, Res, Resource},
};

pub fn set_mouse_hover<TCastRay: CastRay + Resource>(
	mut commands: Commands,
	cam_ray: Option<Res<CamRay>>,
	ray_caster: Res<TCastRay>,
	roots: Query<&ColliderRoot>,
) {
	let mouse_hover = match ray_cast(cam_ray, ray_caster) {
		Some((collider, ..)) => MouseHover(Some(ColliderInfo {
			collider,
			root: get_root(roots, collider),
		})),
		_ => MouseHover::default(),
	};

	commands.insert_resource(mouse_hover);
}

fn ray_cast<TCastRay: CastRay + Resource>(
	cam_ray: Option<Res<CamRay>>,
	ray_caster: Res<TCastRay>,
) -> Option<(Entity, TimeOfImpact)> {
	ray_caster.cast_ray(cam_ray?.0?)
}

fn get_root(roots: Query<&ColliderRoot>, entity: Entity) -> Option<Entity> {
	roots.get(entity).map(|r| r.0).ok()
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{resources::MouseHover, traits::cast_ray::TimeOfImpact};
	use bevy::{
		app::{App, Update},
		ecs::entity::Entity,
		math::{Ray, Vec3},
	};
	use mockall::{automock, predicate::eq};

	#[derive(Resource, Default)]
	struct _CastRay {
		pub mock: Mock_CastRay,
	}

	#[automock]
	impl CastRay for _CastRay {
		fn cast_ray(&self, ray: Ray) -> Option<(Entity, TimeOfImpact)> {
			self.mock.cast_ray(ray)
		}
	}

	fn setup(ray: Option<Ray>) -> App {
		let mut app = App::new();

		app.init_resource::<_CastRay>();
		app.insert_resource(CamRay(ray));
		app.add_systems(Update, set_mouse_hover::<_CastRay>);
		app
	}

	const TEST_RAY: Ray = Ray {
		origin: Vec3::new(5., 6., 7.),
		direction: Vec3::new(11., 12., 13.),
	};

	#[test]
	fn add_target_collider() {
		let mut app = setup(Some(TEST_RAY));
		let collider = app.world.spawn_empty().id();
		let mut cast_ray = app.world.resource_mut::<_CastRay>();
		cast_ray
			.mock
			.expect_cast_ray()
			.return_const((collider, TimeOfImpact(0.)));

		app.update();

		let mouse_hover = app.world.get_resource::<MouseHover<Entity>>();

		assert_eq!(
			Some(collider),
			mouse_hover
				.and_then(|mh| mh.0.clone())
				.map(|ci| ci.collider)
		);
	}

	#[test]
	fn add_target_root() {
		let mut app = setup(Some(TEST_RAY));
		let root = app.world.spawn_empty().id();
		let collider = app.world.spawn(ColliderRoot(root)).id();
		let mut cast_ray = app.world.resource_mut::<_CastRay>();
		cast_ray
			.mock
			.expect_cast_ray()
			.return_const((collider, TimeOfImpact(0.)));

		app.update();

		let mouse_hover = app.world.get_resource::<MouseHover<Entity>>();

		assert_eq!(
			Some(Some(root)),
			mouse_hover.and_then(|mh| mh.0.clone()).map(|ci| ci.root)
		);
	}

	#[test]
	fn set_mouse_hover_none_when_no_collision() {
		let mut app = setup(Some(TEST_RAY));
		let mut cast_ray = app.world.resource_mut::<_CastRay>();
		cast_ray.mock.expect_cast_ray().return_const(None);

		app.update();

		let mouse_hover = app.world.get_resource::<MouseHover<Entity>>();

		assert_eq!(Some(&MouseHover(None)), mouse_hover);
	}

	#[test]
	fn set_mouse_hover_none_when_no_ray() {
		let mut app = setup(None);
		let collider = app.world.spawn_empty().id();
		let mut cast_ray = app.world.resource_mut::<_CastRay>();
		cast_ray
			.mock
			.expect_cast_ray()
			.return_const((collider, TimeOfImpact(0.)));

		app.update();

		let mouse_hover = app.world.get_resource::<MouseHover<Entity>>();

		assert_eq!(Some(&MouseHover(None)), mouse_hover);
	}

	#[test]
	fn call_cast_ray_with_parameters() {
		let mut app = setup(Some(TEST_RAY));
		let mut cast_ray = app.world.resource_mut::<_CastRay>();

		cast_ray
			.mock
			.expect_cast_ray()
			.times(1)
			.with(eq(TEST_RAY))
			.return_const(None);

		app.update();
	}

	#[test]
	fn no_panic_when_cam_ray_missing() {
		let mut app = App::new();
		let mut cast_ray = _CastRay::default();
		cast_ray.mock.expect_cast_ray().return_const(None);
		app.insert_resource(cast_ray);
		app.add_systems(Update, set_mouse_hover::<_CastRay>);

		app.update();
	}
}
