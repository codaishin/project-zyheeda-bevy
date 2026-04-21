use crate::{components::offset::ComputeOffsetTranslation, system_params::ray_caster::RayCaster};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::traits::handles_physics::{
	HoverMode,
	MouseHover,
	MouseHoversOver,
	Raycast,
	SolidObjects,
	Terrain,
	TimeOfImpact,
};

impl<T> Raycast<MouseHover> for RayCaster<'_, '_, T>
where
	T: SystemParam + 'static,
	Self: Raycast<SolidObjects> + Raycast<Terrain>,
{
	fn raycast(&mut self, mouse_hover: MouseHover) -> Option<MouseHoversOver> {
		let cam = self.world_cams.single().ok()?;
		let ray = cam.ray?;

		if let Some(cached) = cam.mouse_hover.get(&mouse_hover) {
			return Some(*cached);
		}

		let object_hit = self.raycast(SolidObjects {
			ray,
			exclude: mouse_hover.exclude.clone(),
			only_hoverable: true,
		});
		let plane_hit = match mouse_hover.mode {
			HoverMode::ColliderOrTerrain => None,
			HoverMode::ColliderOrDirectionFrom(entity) => self.hit_horizontal_plane(ray, entity),
		};
		let ground_hit = self.raycast(Terrain { ray });
		let hover = match (object_hit, plane_hit, ground_hit) {
			(None, None, None) => return None,
			(None, Some(toi), _) | (None, _, Some(toi)) => MouseHoversOver::Point(point(ray, *toi)),
			(Some(object), Some(toi), Some(ground_toi)) if object.time_of_impact > *ground_toi => {
				MouseHoversOver::Point(point(ray, *toi))
			}
			(Some(object), None, Some(toi)) if object.time_of_impact > *toi => {
				MouseHoversOver::Point(point(ray, *toi))
			}
			(Some(object), ..) => MouseHoversOver::Object {
				entity: object.entity,
				point: point(ray, object.time_of_impact),
			},
		};

		let mut cam = self.world_cams.single_mut().ok()?;
		cam.mouse_hover.insert(mouse_hover.clone(), hover);

		Some(hover)
	}
}

impl<T> RayCaster<'_, '_, T>
where
	T: SystemParam,
{
	fn hit_horizontal_plane(&self, ray: Ray3d, entity: Entity) -> Option<TimeOfImpact> {
		let Ok((transform, offset)) = self.transforms.get(entity) else {
			return None;
		};

		let plane_origin = offset.compute_translation(transform);
		let plane = InfinitePlane3d { normal: Dir3::Y };

		ray.intersect_plane(plane_origin, plane)
			.and_then(|toi| TimeOfImpact::try_from_f32(toi).ok())
	}
}

fn point(ray: Ray3d, toi: f32) -> Vec3 {
	ray.origin + ray.direction * toi
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::components::world_camera::WorldCamera;
	use bevy::{
		app::{App, Update},
		ecs::{
			resource::Resource,
			system::{RunSystemError, RunSystemOnce},
		},
	};
	use common::{
		toi,
		tools::Units,
		traits::handles_physics::{HoverMode, RaycastHit, TimeOfImpact},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::collections::HashMap;
	use testing::{NestedMocks, SingleThreadedApp, fake_entity};

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
	impl Raycast<Terrain> for _Ground {
		fn raycast(&mut self, args: Terrain) -> Option<TimeOfImpact> {
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

	impl Raycast<Terrain> for _RayCaster<'_, '_> {
		fn raycast(&mut self, args: Terrain) -> Option<TimeOfImpact> {
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

	mod terrain_mode {
		use super::*;

		#[test]
		fn return_object_hit() -> Result<(), RunSystemError> {
			let ray = Ray3d {
				origin: Vec3::new(1., 2., 3.),
				direction: Dir3::NEG_Y,
			};
			let exclude = fake_entity!(444);
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
							entity: fake_entity!(123),
							time_of_impact: 42.,
						});
				}),
				_Ground::new().with_mock(|mock| {
					mock.expect_raycast()
						.with(eq(Terrain { ray }))
						.return_const(toi!(44.));
				}),
			);

			app.world_mut()
				.run_system_once(move |mut ray_caster: _RayCaster| {
					let hit = ray_caster.raycast(MouseHover {
						exclude: vec![exclude],
						mode: HoverMode::ColliderOrTerrain,
					});

					assert_eq!(
						Some(MouseHoversOver::Object {
							entity: fake_entity!(123),
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
			let exclude = fake_entity!(444);
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
						.with(eq(Terrain { ray }))
						.return_const(toi!(44.));
				}),
			);

			app.world_mut()
				.run_system_once(move |mut ray_caster: _RayCaster| {
					let hit = ray_caster.raycast(MouseHover {
						exclude: vec![exclude],
						mode: HoverMode::ColliderOrTerrain,
					});

					assert_eq!(Some(MouseHoversOver::Point(Vec3::new(1., -42., 3.))), hit,);
				})
		}

		#[test]
		fn return_ground_hit_when_no_object_further_away_than_ground() -> Result<(), RunSystemError>
		{
			let ray = Ray3d {
				origin: Vec3::new(1., 2., 3.),
				direction: Dir3::NEG_Y,
			};
			let exclude = fake_entity!(444);
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
							entity: fake_entity!(123),
							time_of_impact: 100.,
						});
				}),
				_Ground::new().with_mock(|mock| {
					mock.expect_raycast()
						.with(eq(Terrain { ray }))
						.return_const(toi!(44.));
				}),
			);

			app.world_mut()
				.run_system_once(move |mut ray_caster: _RayCaster| {
					let hit = ray_caster.raycast(MouseHover {
						exclude: vec![exclude],
						mode: HoverMode::ColliderOrTerrain,
					});

					assert_eq!(Some(MouseHoversOver::Point(Vec3::new(1., -42., 3.))), hit,);
				})
		}
	}

	mod direction_mode {
		use super::*;
		use crate::components::offset::AimOffset;

		#[test]
		fn return_direction_hit() -> Result<(), RunSystemError> {
			let ray = Ray3d {
				origin: Vec3::new(1., 20., 3.),
				direction: Dir3::NEG_Y,
			};
			let exclude = fake_entity!(444);
			let (mut app, _) = setup(
				ray,
				_Objects::new().with_mock(|mock| {
					mock.expect_raycast().return_const(None);
				}),
				_Ground::new().with_mock(|mock| {
					mock.expect_raycast().return_const(None);
				}),
			);
			let entity = app
				.world_mut()
				.spawn(GlobalTransform::from_xyz(0., 10., 0.))
				.id();

			app.world_mut()
				.run_system_once(move |mut ray_caster: _RayCaster| {
					let hit = ray_caster.raycast(MouseHover {
						exclude: vec![exclude],
						mode: HoverMode::ColliderOrDirectionFrom(entity),
					});

					assert_eq!(Some(MouseHoversOver::Point(Vec3::new(1., 10., 3.))), hit);
				})
		}

		#[test]
		fn return_direction_hit_with_aim_offset() -> Result<(), RunSystemError> {
			let ray = Ray3d {
				origin: Vec3::new(1., 20., 3.),
				direction: Dir3::NEG_Y,
			};
			let exclude = fake_entity!(444);
			let (mut app, _) = setup(
				ray,
				_Objects::new().with_mock(|mock| {
					mock.expect_raycast().return_const(None);
				}),
				_Ground::new().with_mock(|mock| {
					mock.expect_raycast().return_const(None);
				}),
			);
			let entity = app
				.world_mut()
				.spawn((GlobalTransform::from_xyz(0., 10., 0.), AimOffset(1.)))
				.id();

			app.world_mut()
				.run_system_once(move |mut ray_caster: _RayCaster| {
					let hit = ray_caster.raycast(MouseHover {
						exclude: vec![exclude],
						mode: HoverMode::ColliderOrDirectionFrom(entity),
					});

					assert_eq!(Some(MouseHoversOver::Point(Vec3::new(1., 11., 3.))), hit);
				})
		}

		#[test]
		fn return_direction_hit_when_ground_hit() -> Result<(), RunSystemError> {
			let ray = Ray3d {
				origin: Vec3::new(1., 20., 3.),
				direction: Dir3::NEG_Y,
			};
			let exclude = fake_entity!(444);
			let (mut app, _) = setup(
				ray,
				_Objects::new().with_mock(|mock| {
					mock.expect_raycast().return_const(None);
				}),
				_Ground::new().with_mock(|mock| {
					mock.expect_raycast()
						.return_const(Some(TimeOfImpact::from(Units::from(1.))));
				}),
			);
			let entity = app
				.world_mut()
				.spawn(GlobalTransform::from_xyz(0., 10., 0.))
				.id();

			app.world_mut()
				.run_system_once(move |mut ray_caster: _RayCaster| {
					let hit = ray_caster.raycast(MouseHover {
						exclude: vec![exclude],
						mode: HoverMode::ColliderOrDirectionFrom(entity),
					});

					assert_eq!(Some(MouseHoversOver::Point(Vec3::new(1., 10., 3.))), hit);
				})
		}

		#[test]
		fn return_object_hit_when_ground_hit_further_away() -> Result<(), RunSystemError> {
			let ray = Ray3d {
				origin: Vec3::new(1., 20., 3.),
				direction: Dir3::NEG_Y,
			};
			let exclude = fake_entity!(444);
			let (mut app, _) = setup(
				ray,
				_Objects::new().with_mock(|mock| {
					mock.expect_raycast().return_const(Some(RaycastHit {
						entity: fake_entity!(555),
						time_of_impact: 12.,
					}));
				}),
				_Ground::new().with_mock(|mock| {
					mock.expect_raycast()
						.return_const(Some(TimeOfImpact::from(Units::from(15.))));
				}),
			);
			let entity = app
				.world_mut()
				.spawn(GlobalTransform::from_xyz(0., 10., 0.))
				.id();

			app.world_mut()
				.run_system_once(move |mut ray_caster: _RayCaster| {
					let hit = ray_caster.raycast(MouseHover {
						exclude: vec![exclude],
						mode: HoverMode::ColliderOrDirectionFrom(entity),
					});

					assert_eq!(
						Some(MouseHoversOver::Object {
							entity: fake_entity!(555),
							point: Vec3::new(1., 8., 3.)
						}),
						hit
					);
				})
		}

		#[test]
		fn return_direction_hit_when_object_hit_further_away_than_ground_hit()
		-> Result<(), RunSystemError> {
			let ray = Ray3d {
				origin: Vec3::new(1., 20., 3.),
				direction: Dir3::NEG_Y,
			};
			let exclude = fake_entity!(444);
			let (mut app, _) = setup(
				ray,
				_Objects::new().with_mock(|mock| {
					mock.expect_raycast().return_const(Some(RaycastHit {
						entity: fake_entity!(555),
						time_of_impact: 15.,
					}));
				}),
				_Ground::new().with_mock(|mock| {
					mock.expect_raycast()
						.return_const(Some(TimeOfImpact::from(Units::from(12.))));
				}),
			);
			let entity = app
				.world_mut()
				.spawn(GlobalTransform::from_xyz(0., 10., 0.))
				.id();

			app.world_mut()
				.run_system_once(move |mut ray_caster: _RayCaster| {
					let hit = ray_caster.raycast(MouseHover {
						exclude: vec![exclude],
						mode: HoverMode::ColliderOrDirectionFrom(entity),
					});

					assert_eq!(Some(MouseHoversOver::Point(Vec3::new(1., 10., 3.))), hit);
				})
		}
	}

	mod cache {
		use super::*;

		#[test]
		fn return_cached_mouse_hover() -> Result<(), RunSystemError> {
			let ray = Ray3d {
				origin: Vec3::new(1., 2., 3.),
				direction: Dir3::NEG_Y,
			};
			let exclude = fake_entity!(444);
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
				MouseHover {
					exclude: vec![exclude],
					mode: HoverMode::ColliderOrTerrain,
				},
				MouseHoversOver::Object {
					entity: fake_entity!(321),
					point: Vec3::new(4., 11., 2.),
				},
			);

			app.world_mut()
				.run_system_once(move |mut ray_caster: _RayCaster| {
					let hit = ray_caster.raycast(MouseHover {
						exclude: vec![exclude],
						mode: HoverMode::ColliderOrTerrain,
					});

					assert_eq!(
						Some(MouseHoversOver::Object {
							entity: fake_entity!(321),
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
			let exclude = fake_entity!(444);
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
							entity: fake_entity!(123),
							time_of_impact: 42.,
						});
				}),
				_Ground::new().with_mock(|mock| {
					mock.expect_raycast()
						.with(eq(Terrain { ray }))
						.return_const(toi!(44.));
				}),
			);

			app.world_mut()
				.run_system_once(move |mut ray_caster: _RayCaster| {
					ray_caster.raycast(MouseHover {
						exclude: vec![exclude],
						mode: HoverMode::ColliderOrTerrain,
					});
				})?;

			assert_eq!(
				Some(&WorldCamera {
					mouse_hover: HashMap::from([(
						MouseHover {
							exclude: vec![exclude],
							mode: HoverMode::ColliderOrTerrain,
						},
						MouseHoversOver::Object {
							entity: fake_entity!(123),
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
}
