use crate::resources::{cam_ray::CamRay, mouse_hover::MouseHover};
use bevy::{
	ecs::{
		entity::Entity,
		query::With,
		system::{Commands, Query, Res},
	},
	math::Ray3d,
};
use bevy_rapier3d::plugin::ReadRapierContext;
use common::{
	components::{NoTarget, collider_relations::ChildColliderOf},
	tools::collider_info::ColliderInfo,
	traits::cast_ray::{CastRay, GetRayCaster, TimeOfImpact},
};
use std::ops::Deref;

pub(crate) fn set_mouse_hover(
	get_ray_caster: ReadRapierContext,
	commands: Commands,
	cam_ray: Option<Res<CamRay>>,
	roots: Query<&ChildColliderOf>,
	non_target_ables: Query<(), With<NoTarget>>,
) {
	internal_set_mouse_hover(get_ray_caster, commands, cam_ray, roots, non_target_ables)
}

fn internal_set_mouse_hover<TGetRayCaster>(
	get_ray_caster: TGetRayCaster,
	mut commands: Commands,
	cam_ray: Option<Res<CamRay>>,
	roots: Query<&ChildColliderOf>,
	non_target_ables: Query<(), With<NoTarget>>,
) where
	TGetRayCaster: GetRayCaster<Ray3d>,
{
	let Ok(ray_caster) = get_ray_caster.get_ray_caster() else {
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
	roots: Query<&ChildColliderOf>,
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
	ray_caster: TCastRay,
) -> Option<(Entity, TimeOfImpact)> {
	let &CamRay(Some(cam_ray)) = cam_ray?.deref() else {
		return None;
	};
	ray_caster.cast_ray(&cam_ray)
}

fn get_root(entity: Entity, roots: Query<&ChildColliderOf>) -> Option<Entity> {
	roots.get(entity).map(|ChildColliderOf(r)| *r).ok()
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::{
			entity::Entity,
			system::{In, RunSystemError, RunSystemOnce},
		},
		math::{Ray3d, Vec3},
	};
	use common::{
		components::NoTarget,
		test_tools::utils::SingleThreadedApp,
		traits::{cast_ray::TimeOfImpact, nested_mock::NestedMocks},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(NestedMocks)]
	struct _GetRayCaster {
		mock: Mock_GetRayCaster,
	}

	pub enum _GetRayCasterError {}

	#[automock]
	impl GetRayCaster<Ray3d> for _GetRayCaster {
		type TError = _GetRayCasterError;
		type TRayCaster<'a>
			= _RayCaster
		where
			Self: 'a;

		fn get_ray_caster(&self) -> Result<Self::TRayCaster<'_>, Self::TError> {
			self.mock.get_ray_caster()
		}
	}

	#[derive(NestedMocks)]
	pub struct _RayCaster {
		pub mock: Mock_RayCaster,
	}

	#[automock]
	impl CastRay<Ray3d> for _RayCaster {
		fn cast_ray(&self, ray: &Ray3d) -> Option<(Entity, TimeOfImpact)> {
			self.mock.cast_ray(ray)
		}
	}

	fn setup(ray: Option<Ray3d>) -> App {
		let mut app = App::new();

		app.insert_resource(CamRay(ray));

		app
	}

	fn test_ray() -> Option<Ray3d> {
		Some(Ray3d {
			origin: Vec3::new(5., 6., 7.),
			direction: Vec3::new(11., 12., 13.).try_into().unwrap(),
		})
	}

	#[test]
	fn add_target_collider() -> Result<(), RunSystemError> {
		let mut app = setup(test_ray());
		let collider = app.world_mut().spawn_empty().id();
		let get_ray_caster = _GetRayCaster::new().with_mock(move |mock| {
			mock.expect_get_ray_caster().times(1).returning(move || {
				Ok(_RayCaster::new().with_mock(move |mock| {
					mock.expect_cast_ray()
						.times(1)
						.return_const((collider, TimeOfImpact(0.)));
				}))
			});
		});

		app.world_mut().run_system_once_with(
			internal_set_mouse_hover::<In<_GetRayCaster>>,
			get_ray_caster,
		)?;

		let mouse_hover = app.world().get_resource::<MouseHover<Entity>>();

		assert_eq!(
			Some(collider),
			mouse_hover.and_then(|mh| mh.0).map(|ci| ci.collider)
		);
		Ok(())
	}

	#[test]
	fn add_target_root() -> Result<(), RunSystemError> {
		let mut app = setup(test_ray());
		let root = app.world_mut().spawn_empty().id();
		let collider = app.world_mut().spawn(ChildColliderOf(root)).id();
		let get_ray_caster = _GetRayCaster::new().with_mock(move |mock| {
			mock.expect_get_ray_caster().returning(move || {
				Ok(_RayCaster::new().with_mock(move |mock| {
					mock.expect_cast_ray()
						.return_const((collider, TimeOfImpact(0.)));
				}))
			});
		});

		app.world_mut().run_system_once_with(
			internal_set_mouse_hover::<In<_GetRayCaster>>,
			get_ray_caster,
		)?;

		let mouse_hover = app.world().get_resource::<MouseHover<Entity>>();

		assert_eq!(
			Some(Some(root)),
			mouse_hover.and_then(|mh| mh.0).map(|ci| ci.root)
		);
		Ok(())
	}

	#[test]
	fn set_mouse_hover_none_when_no_collision() -> Result<(), RunSystemError> {
		let mut app = setup(test_ray());
		let get_ray_caster = _GetRayCaster::new().with_mock(move |mock| {
			mock.expect_get_ray_caster().returning(move || {
				Ok(_RayCaster::new().with_mock(move |mock| {
					mock.expect_cast_ray().return_const(None);
				}))
			});
		});

		app.world_mut().run_system_once_with(
			internal_set_mouse_hover::<In<_GetRayCaster>>,
			get_ray_caster,
		)?;

		let mouse_hover = app.world().get_resource::<MouseHover<Entity>>();

		assert_eq!(Some(&MouseHover(None)), mouse_hover);
		Ok(())
	}

	#[test]
	fn set_mouse_hover_none_when_no_ray() -> Result<(), RunSystemError> {
		let mut app = setup(None);
		let collider = app.world_mut().spawn_empty().id();
		let get_ray_caster = _GetRayCaster::new().with_mock(move |mock| {
			mock.expect_get_ray_caster().returning(move || {
				Ok(_RayCaster::new().with_mock(move |mock| {
					mock.expect_cast_ray()
						.return_const((collider, TimeOfImpact(0.)));
				}))
			});
		});

		app.world_mut().run_system_once_with(
			internal_set_mouse_hover::<In<_GetRayCaster>>,
			get_ray_caster,
		)?;

		let mouse_hover = app.world().get_resource::<MouseHover<Entity>>();

		assert_eq!(Some(&MouseHover(None)), mouse_hover);
		Ok(())
	}

	#[test]
	fn set_mouse_hover_none_when_collider_root_marked_as_no_target() -> Result<(), RunSystemError> {
		let mut app = setup(test_ray());
		let root = app.world_mut().spawn(NoTarget).id();
		let collider = app.world_mut().spawn(ChildColliderOf(root)).id();
		let get_ray_caster = _GetRayCaster::new().with_mock(move |mock| {
			mock.expect_get_ray_caster().returning(move || {
				Ok(_RayCaster::new().with_mock(move |mock| {
					mock.expect_cast_ray()
						.return_const((collider, TimeOfImpact(0.)));
				}))
			});
		});

		app.world_mut().run_system_once_with(
			internal_set_mouse_hover::<In<_GetRayCaster>>,
			get_ray_caster,
		)?;

		let mouse_hover = app.world().get_resource::<MouseHover<Entity>>();

		assert_eq!(Some(&MouseHover::default()), mouse_hover);
		Ok(())
	}

	#[test]
	fn set_mouse_hover_none_when_collider_marked_as_no_target() -> Result<(), RunSystemError> {
		let mut app = setup(test_ray());
		let collider = app.world_mut().spawn(NoTarget).id();
		let get_ray_caster = _GetRayCaster::new().with_mock(move |mock| {
			mock.expect_get_ray_caster().returning(move || {
				Ok(_RayCaster::new().with_mock(move |mock| {
					mock.expect_cast_ray()
						.return_const((collider, TimeOfImpact(0.)));
				}))
			});
		});

		app.world_mut().run_system_once_with(
			internal_set_mouse_hover::<In<_GetRayCaster>>,
			get_ray_caster,
		)?;

		let mouse_hover = app.world().get_resource::<MouseHover<Entity>>();

		assert_eq!(Some(&MouseHover::default()), mouse_hover);
		Ok(())
	}

	#[test]
	fn call_cast_ray_with_parameters() -> Result<(), RunSystemError> {
		let mut app = setup(test_ray());
		let get_ray_caster = _GetRayCaster::new().with_mock(move |mock| {
			mock.expect_get_ray_caster().returning(move || {
				Ok(_RayCaster::new().with_mock(move |mock| {
					mock.expect_cast_ray()
						.times(1)
						.with(eq(test_ray().unwrap()))
						.return_const(None);
				}))
			});
		});

		app.world_mut().run_system_once_with(
			internal_set_mouse_hover::<In<_GetRayCaster>>,
			get_ray_caster,
		)?;
		Ok(())
	}

	#[test]
	fn no_panic_when_cam_ray_missing() -> Result<(), RunSystemError> {
		let mut app = App::new().single_threaded(Update);
		let get_ray_caster = _GetRayCaster::new().with_mock(move |mock| {
			mock.expect_get_ray_caster().returning(move || {
				Ok(_RayCaster::new().with_mock(move |mock| {
					mock.expect_cast_ray().return_const(None);
				}))
			});
		});

		app.world_mut().run_system_once_with(
			internal_set_mouse_hover::<In<_GetRayCaster>>,
			get_ray_caster,
		)?;
		Ok(())
	}
}
