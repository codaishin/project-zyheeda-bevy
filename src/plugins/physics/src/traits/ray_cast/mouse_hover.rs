use crate::traits::ray_cast::RayCaster;
use bevy::{
	ecs::system::SystemParam,
	math::{Ray3d, Vec3},
};
use common::traits::handles_physics::{
	Ground,
	MouseHover,
	MouseHoversOver,
	Raycast,
	SolidObjects,
	TimeOfImpact,
};

impl<T> Raycast<MouseHover> for RayCaster<'_, '_, T>
where
	T: SystemParam + 'static,
	Self: Raycast<SolidObjects> + Raycast<Ground>,
{
	fn raycast(&mut self, MouseHover { exclude }: MouseHover) -> Option<MouseHoversOver> {
		let cam = self.world_cams.single_mut().ok()?;
		let ray = cam.ray?;

		if let Some(cached) = cam.mouse_hover.get(&exclude) {
			return Some(*cached);
		}

		let object_hit = self.raycast(SolidObjects {
			ray,
			exclude: exclude.clone(),
			only_hoverable: true,
		});
		let ground_hit = self.raycast(Ground { ray });
		let hover = match (object_hit, ground_hit) {
			(None, None) => return None,
			(None, Some(TimeOfImpact(time_of_impact))) => MouseHoversOver::Ground {
				point: point(ray, time_of_impact),
			},
			(Some(object), Some(TimeOfImpact(ground_time_of_impact)))
				if object.time_of_impact > ground_time_of_impact =>
			{
				MouseHoversOver::Ground {
					point: point(ray, ground_time_of_impact),
				}
			}
			(Some(object), _) => MouseHoversOver::Object {
				entity: object.entity,
				point: point(ray, object.time_of_impact),
			},
		};

		let mut cam = self.world_cams.single_mut().ok()?;
		cam.mouse_hover.insert(exclude, hover);

		Some(hover)
	}
}

fn point(ray: Ray3d, toi: f32) -> Vec3 {
	ray.origin + ray.direction * toi
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::world_camera::WorldCamera;
	use bevy::{
		app::{App, Update},
		ecs::{
			resource::Resource,
			system::{RunSystemError, RunSystemOnce},
		},
		prelude::*,
	};
	use common::traits::handles_physics::RaycastHit;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::collections::HashMap;
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Resource, NestedMocks)]
	struct _Objects {
		mock: Mock_Objects,
	}

	#[automock]
	impl Raycast<SolidObjects> for _Objects {
		fn raycast(&mut self, args: SolidObjects) -> Option<RaycastHit> {
			self.mock.raycast(args)
		}
	}

	#[derive(Resource, NestedMocks)]
	struct _Ground {
		mock: Mock_Ground,
	}

	#[automock]
	impl Raycast<Ground> for _Ground {
		fn raycast(&mut self, args: Ground) -> Option<TimeOfImpact> {
			self.mock.raycast(args)
		}
	}

	type _RayCaster<'w, 's> =
		RayCaster<'w, 's, (ResMut<'static, _Objects>, ResMut<'static, _Ground>)>;

	impl Raycast<SolidObjects> for _RayCaster<'_, '_> {
		fn raycast(&mut self, args: SolidObjects) -> Option<RaycastHit> {
			self.context.0.raycast(args)
		}
	}

	impl Raycast<Ground> for _RayCaster<'_, '_> {
		fn raycast(&mut self, args: Ground) -> Option<TimeOfImpact> {
			self.context.1.raycast(args)
		}
	}

	fn setup(ray: Ray3d, objects: _Objects, ground: _Ground) -> (App, Entity) {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(objects);
		app.insert_resource(ground);

		let cam = app
			.world_mut()
			.spawn(WorldCamera {
				ray: Some(ray),
				..default()
			})
			.id();

		(app, cam)
	}

	#[test]
	fn return_object_hit() -> Result<(), RunSystemError> {
		let ray = Ray3d {
			origin: Vec3::new(1., 2., 3.),
			direction: Dir3::NEG_Y,
		};
		let exclude = Entity::from_raw(444);
		let (mut app, _) = setup(
			ray,
			_Objects::new().with_mock(|mock| {
				mock.expect_raycast()
					.with(eq(SolidObjects {
						ray,
						exclude: vec![exclude],
						only_hoverable: true,
					}))
					.return_const(RaycastHit {
						entity: Entity::from_raw(123),
						time_of_impact: 42.,
					});
			}),
			_Ground::new().with_mock(|mock| {
				mock.expect_raycast()
					.with(eq(Ground { ray }))
					.return_const(TimeOfImpact(44.));
			}),
		);

		app.world_mut()
			.run_system_once(move |mut ray_caster: _RayCaster| {
				let hit = ray_caster.raycast(MouseHover {
					exclude: vec![exclude],
				});

				assert_eq!(
					Some(MouseHoversOver::Object {
						entity: Entity::from_raw(123),
						point: Vec3::new(1., -40., 3.),
					}),
					hit,
				);
			})
	}

	#[test]
	fn return_ground_hit_when_no_object_hit() -> Result<(), RunSystemError> {
		let ray = Ray3d {
			origin: Vec3::new(1., 2., 3.),
			direction: Dir3::NEG_Y,
		};
		let exclude = Entity::from_raw(444);
		let (mut app, _) = setup(
			ray,
			_Objects::new().with_mock(|mock| {
				mock.expect_raycast()
					.with(eq(SolidObjects {
						ray,
						exclude: vec![exclude],
						only_hoverable: true,
					}))
					.return_const(None);
			}),
			_Ground::new().with_mock(|mock| {
				mock.expect_raycast()
					.with(eq(Ground { ray }))
					.return_const(TimeOfImpact(44.));
			}),
		);

		app.world_mut()
			.run_system_once(move |mut ray_caster: _RayCaster| {
				let hit = ray_caster.raycast(MouseHover {
					exclude: vec![exclude],
				});

				assert_eq!(
					Some(MouseHoversOver::Ground {
						point: Vec3::new(1., -42., 3.)
					}),
					hit,
				);
			})
	}

	#[test]
	fn return_ground_hit_when_no_object_further_away_than_ground() -> Result<(), RunSystemError> {
		let ray = Ray3d {
			origin: Vec3::new(1., 2., 3.),
			direction: Dir3::NEG_Y,
		};
		let exclude = Entity::from_raw(444);
		let (mut app, _) = setup(
			ray,
			_Objects::new().with_mock(|mock| {
				mock.expect_raycast()
					.with(eq(SolidObjects {
						ray,
						exclude: vec![exclude],
						only_hoverable: true,
					}))
					.return_const(RaycastHit {
						entity: Entity::from_raw(123),
						time_of_impact: 100.,
					});
			}),
			_Ground::new().with_mock(|mock| {
				mock.expect_raycast()
					.with(eq(Ground { ray }))
					.return_const(TimeOfImpact(44.));
			}),
		);

		app.world_mut()
			.run_system_once(move |mut ray_caster: _RayCaster| {
				let hit = ray_caster.raycast(MouseHover {
					exclude: vec![exclude],
				});

				assert_eq!(
					Some(MouseHoversOver::Ground {
						point: Vec3::new(1., -42., 3.)
					}),
					hit,
				);
			})
	}

	#[test]
	fn return_cached_mouse_hover() -> Result<(), RunSystemError> {
		let ray = Ray3d {
			origin: Vec3::new(1., 2., 3.),
			direction: Dir3::NEG_Y,
		};
		let exclude = Entity::from_raw(444);
		let (mut app, cam) = setup(
			ray,
			_Objects::new().with_mock(|mock| {
				mock.expect_raycast().never();
			}),
			_Ground::new().with_mock(|mock| {
				mock.expect_raycast().never();
			}),
		);
		let mut cam = app.world_mut().entity_mut(cam);
		let mut cam = cam.get_mut::<WorldCamera>().unwrap();
		cam.mouse_hover.insert(
			vec![exclude],
			MouseHoversOver::Object {
				entity: Entity::from_raw(321),
				point: Vec3::new(4., 11., 2.),
			},
		);

		app.world_mut()
			.run_system_once(move |mut ray_caster: _RayCaster| {
				let hit = ray_caster.raycast(MouseHover {
					exclude: vec![exclude],
				});

				assert_eq!(
					Some(MouseHoversOver::Object {
						entity: Entity::from_raw(321),
						point: Vec3::new(4., 11., 2.),
					}),
					hit,
				);
			})
	}

	#[test]
	fn store_new_hit_in_world_camera() -> Result<(), RunSystemError> {
		let ray = Ray3d {
			origin: Vec3::new(1., 2., 3.),
			direction: Dir3::NEG_Y,
		};
		let exclude = Entity::from_raw(444);
		let (mut app, cam) = setup(
			ray,
			_Objects::new().with_mock(|mock| {
				mock.expect_raycast()
					.with(eq(SolidObjects {
						ray,
						exclude: vec![exclude],
						only_hoverable: true,
					}))
					.return_const(RaycastHit {
						entity: Entity::from_raw(123),
						time_of_impact: 42.,
					});
			}),
			_Ground::new().with_mock(|mock| {
				mock.expect_raycast()
					.with(eq(Ground { ray }))
					.return_const(TimeOfImpact(44.));
			}),
		);

		app.world_mut()
			.run_system_once(move |mut ray_caster: _RayCaster| {
				ray_caster.raycast(MouseHover {
					exclude: vec![exclude],
				});
			})?;

		assert_eq!(
			Some(&WorldCamera {
				mouse_hover: HashMap::from([(
					vec![exclude],
					MouseHoversOver::Object {
						entity: Entity::from_raw(123),
						point: Vec3::new(1., -40., 3.),
					}
				)]),
				ray: Some(ray),
			}),
			app.world().entity(cam).get::<WorldCamera>(),
		);
		Ok(())
	}
}
