use std::ops::Deref;

use crate::resources::{cam_ray::CamRay, mouse_hover::MouseHover};
use bevy::{
	ecs::{
		entity::Entity,
		query::With,
		system::{Commands, Query, Res},
	},
	math::Ray3d,
	prelude::Component,
};
use common::{
	components::{ColliderRoot, NoTarget},
	tools::collider_info::ColliderInfo,
	traits::cast_ray::{CastRay, TimeOfImpact},
};

pub(crate) fn set_mouse_hover<TCastRay: CastRay<Ray3d> + Component>(
	mut commands: Commands,
	cam_ray: Option<Res<CamRay>>,
	ray_caster: Query<&TCastRay>,
	roots: Query<&ColliderRoot>,
	non_target_ables: Query<(), With<NoTarget>>,
) {
	let Ok(ray_caster) = ray_caster.get_single() else {
		return;
	};
	let mouse_hover = match ray_cast(cam_ray, ray_caster) {
		Some((collider, ..)) => get_mouse_hover(collider, roots, non_target_ables),
		_ => MouseHover::default(),
	};

	commands.insert_resource(mouse_hover);
}

fn get_mouse_hover(
	collider: Entity,
	roots: Query<&ColliderRoot>,
	non_target_ables: Query<(), With<NoTarget>>,
) -> MouseHover {
	if non_target_ables.contains(collider) {
		return MouseHover(None);
	}

	match get_root(collider, roots) {
		Some(root) if non_target_ables.contains(root) => MouseHover(None),
		root => MouseHover(Some(ColliderInfo { collider, root })),
	}
}

fn ray_cast<TCastRay: CastRay<Ray3d>>(
	cam_ray: Option<Res<CamRay>>,
	ray_caster: &TCastRay,
) -> Option<(Entity, TimeOfImpact)> {
	let &CamRay(Some(cam_ray)) = cam_ray?.deref() else {
		return None;
	};
	ray_caster.cast_ray(&cam_ray)
}

fn get_root(entity: Entity, roots: Query<&ColliderRoot>) -> Option<Entity> {
	roots.get(entity).map(|ColliderRoot(r)| *r).ok()
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::entity::Entity,
		math::{Ray3d, Vec3},
	};
	use common::{
		components::NoTarget,
		traits::{cast_ray::TimeOfImpact, nested_mock::NestedMocks},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Component, NestedMocks)]
	struct _CastRay {
		pub mock: Mock_CastRay,
	}

	#[automock]
	impl CastRay<Ray3d> for _CastRay {
		fn cast_ray(&self, ray: &Ray3d) -> Option<(Entity, TimeOfImpact)> {
			self.mock.cast_ray(ray)
		}
	}

	fn setup(ray: Option<Ray3d>) -> App {
		let mut app = App::new();

		app.insert_resource(CamRay(ray));
		app.add_systems(Update, set_mouse_hover::<_CastRay>);
		app
	}

	fn test_ray() -> Option<Ray3d> {
		Some(Ray3d {
			origin: Vec3::new(5., 6., 7.),
			direction: Vec3::new(11., 12., 13.).try_into().unwrap(),
		})
	}

	#[test]
	fn add_target_collider() {
		let mut app = setup(test_ray());
		let collider = app.world_mut().spawn_empty().id();
		app.world_mut().spawn(_CastRay::new().with_mock(|mock| {
			mock.expect_cast_ray()
				.return_const((collider, TimeOfImpact(0.)));
		}));

		app.update();

		let mouse_hover = app.world().get_resource::<MouseHover<Entity>>();

		assert_eq!(
			Some(collider),
			mouse_hover.and_then(|mh| mh.0).map(|ci| ci.collider)
		);
	}

	#[test]
	fn add_target_root() {
		let mut app = setup(test_ray());
		let root = app.world_mut().spawn_empty().id();
		let collider = app.world_mut().spawn(ColliderRoot(root)).id();
		app.world_mut().spawn(_CastRay::new().with_mock(|mock| {
			mock.expect_cast_ray()
				.return_const((collider, TimeOfImpact(0.)));
		}));

		app.update();

		let mouse_hover = app.world().get_resource::<MouseHover<Entity>>();

		assert_eq!(
			Some(Some(root)),
			mouse_hover.and_then(|mh| mh.0).map(|ci| ci.root)
		);
	}

	#[test]
	fn set_mouse_hover_none_when_no_collision() {
		let mut app = setup(test_ray());
		app.world_mut().spawn(_CastRay::new().with_mock(|mock| {
			mock.expect_cast_ray().return_const(None);
		}));

		app.update();

		let mouse_hover = app.world().get_resource::<MouseHover<Entity>>();

		assert_eq!(Some(&MouseHover(None)), mouse_hover);
	}

	#[test]
	fn set_mouse_hover_none_when_no_ray() {
		let mut app = setup(None);
		let collider = app.world_mut().spawn_empty().id();
		app.world_mut().spawn(_CastRay::new().with_mock(|mock| {
			mock.expect_cast_ray()
				.return_const((collider, TimeOfImpact(0.)));
		}));

		app.update();

		let mouse_hover = app.world().get_resource::<MouseHover<Entity>>();

		assert_eq!(Some(&MouseHover(None)), mouse_hover);
	}

	#[test]
	fn set_mouse_hover_none_when_collider_root_marked_as_no_target() {
		let mut app = setup(test_ray());
		let root = app.world_mut().spawn(NoTarget).id();
		let collider = app.world_mut().spawn(ColliderRoot(root)).id();
		app.world_mut().spawn(_CastRay::new().with_mock(|mock| {
			mock.expect_cast_ray()
				.return_const((collider, TimeOfImpact(0.)));
		}));

		app.update();

		let mouse_hover = app.world().get_resource::<MouseHover<Entity>>();

		assert_eq!(Some(&MouseHover::default()), mouse_hover);
	}

	#[test]
	fn set_mouse_hover_none_when_collider_marked_as_no_target() {
		let mut app = setup(test_ray());
		let collider = app.world_mut().spawn(NoTarget).id();
		app.world_mut().spawn(_CastRay::new().with_mock(|mock| {
			mock.expect_cast_ray()
				.return_const((collider, TimeOfImpact(0.)));
		}));

		app.update();

		let mouse_hover = app.world().get_resource::<MouseHover<Entity>>();

		assert_eq!(Some(&MouseHover::default()), mouse_hover);
	}

	#[test]
	fn call_cast_ray_with_parameters() {
		let mut app = setup(test_ray());
		app.world_mut().spawn(_CastRay::new().with_mock(|mock| {
			mock.expect_cast_ray()
				.times(1)
				.with(eq(test_ray().unwrap()))
				.return_const(None);
		}));

		app.update();
	}

	#[test]
	fn no_panic_when_cam_ray_missing() {
		let mut app = App::new();
		app.world_mut().spawn(_CastRay::new().with_mock(|mock| {
			mock.expect_cast_ray().return_const(None);
		}));
		app.add_systems(Update, set_mouse_hover::<_CastRay>);

		app.update();
	}
}
