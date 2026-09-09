use crate::components::{
	cast_rays::{CastRayFor, CastRays, RayCastResult},
	collider::ColliderShape,
	model::PhysicsModel,
	skill_transform::SkillTransforms,
};
use bevy::prelude::*;
use common::prelude::*;

impl CastRays {
	pub(crate) fn apply_beam_blocks(
		objects: Query<(&CastRays, &SkillTransforms)>,
		mut targets: Query<(
			&mut Transform,
			Option<&ColliderShape>,
			Option<&PhysicsModel>,
		)>,
		mut commands: ZyheedaCommands,
	) {
		for (ray_casts, skill_transforms) in &objects {
			let Some(RayCastResult { toi, .. }) = ray_casts.results.get(&CastRayFor::Beam) else {
				continue;
			};

			for entity in skill_transforms.iter() {
				let Ok((mut transform, collider, model)) = targets.get_mut(entity) else {
					continue;
				};
				let half_length = **toi / 2.;

				// move beam center in the middle of both ends
				transform.translation.z = -half_length;

				// beams are y-aligned cylinders/capsules rotated forward, so we need to scale y direction
				match (collider, model) {
					// update collider shape to trigger reinsertion for immediate collider update
					(Some(ColliderShape::Cylinder { radius, half_y }), None) => {
						if **half_y == half_length {
							continue;
						}
						commands.try_apply_on(&entity, |mut e| {
							e.try_insert(ColliderShape::Cylinder {
								half_y: Units::from(half_length),
								radius: *radius,
							});
						});
					}
					(Some(ColliderShape::Capsule { radius, half_y }), None) => {
						if **half_y == half_length {
							continue;
						}
						commands.try_apply_on(&entity, |mut e| {
							e.try_insert(ColliderShape::Capsule {
								half_y: Units::from(half_length),
								radius: *radius,
							});
						});
					}
					(None, Some(PhysicsModel::Beam { half_y, radius })) => {
						if **half_y == half_length {
							continue;
						}
						commands.try_apply_on(&entity, |mut e| {
							e.try_insert(PhysicsModel::Beam {
								half_y: Units::from(half_length),
								radius: *radius,
							});
						});
					}
					_ => {
						transform.scale.y = **toi;
					}
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::components::skill_transform::SkillTransformOf;
	use bevy::ecs::system::RunSystemError;
	use std::collections::HashMap;
	use testing::{IsChanged, SingleThreadedApp};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			(
				CastRays::apply_beam_blocks,
				IsChanged::<ColliderShape>::detect,
				IsChanged::<PhysicsModel>::detect,
			)
				.chain(),
		);

		app
	}

	mod skill_transforms {
		use super::*;

		#[test]
		fn update_transform_with_toi() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn(CastRays {
					results: HashMap::from([(
						CastRayFor::Beam,
						RayCastResult {
							toi: toi!(11000.),
							..default()
						},
					)]),
				})
				.id();
			let skill_transform = app.world_mut().spawn(SkillTransformOf(entity)).id();

			app.update();

			assert_eq!(
				Some(&Transform {
					translation: Vec3::ZERO.with_z(-5500.),
					scale: Vec3::ONE.with_y(11000.),
					..default()
				}),
				app.world().entity(skill_transform).get::<Transform>(),
			);
		}
	}

	mod colliders {
		use super::*;
		use test_case::test_case;

		#[test_case(
			ColliderShape::Cylinder {
				half_y: Units::from(0.5),
				radius: Units::from(2.),
			},
			ColliderShape::Cylinder {
				half_y: Units::from(5500.),
				radius: Units::from(2.)
			};
			"cylinder"
		)]
		#[test_case(
			ColliderShape::Capsule {
				half_y: Units::from(0.5),
				radius: Units::from(2.),
			},
			ColliderShape::Capsule {
				half_y: Units::from(5500.),
				radius: Units::from(2.)
			};
			"capsule"
		)]
		fn update_cylinder_collider(
			collider: ColliderShape,
			expected: ColliderShape,
		) -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn(CastRays {
					results: HashMap::from([(
						CastRayFor::Beam,
						RayCastResult {
							toi: toi!(11000.),
							..default()
						},
					)]),
				})
				.id();
			let skill_transform = app
				.world_mut()
				.spawn((SkillTransformOf(entity), collider))
				.id();

			app.update();

			assert_eq!(
				(
					Some(&Transform {
						translation: Vec3::ZERO.with_z(-5500.),
						..default()
					}),
					Some(&expected),
				),
				(
					app.world().entity(skill_transform).get::<Transform>(),
					app.world().entity(skill_transform).get::<ColliderShape>(),
				)
			);
			Ok(())
		}

		#[test_case(
			ColliderShape::Cylinder {
				half_y: Units::from(0.5),
				radius: Units::from(2.),
			};
			"cylinder"
		)]
		#[test_case(
			ColliderShape::Capsule {
				half_y: Units::from(0.5),
				radius: Units::from(2.),
			};
			"capsule"
		)]
		fn do_not_update_collider_when_beam_length_did_not_change(
			collider: ColliderShape,
		) -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn(CastRays {
					results: HashMap::from([(
						CastRayFor::Beam,
						RayCastResult {
							toi: toi!(11000.),
							..default()
						},
					)]),
				})
				.id();
			let skill_transform = app
				.world_mut()
				.spawn((SkillTransformOf(entity), collider))
				.id();

			app.update();
			app.update();

			assert_eq!(
				Some(&IsChanged::FALSE),
				app.world()
					.entity(skill_transform)
					.get::<IsChanged<ColliderShape>>(),
			);
			Ok(())
		}
	}

	mod model {
		use super::*;

		#[test]
		fn update_cylinder_collider() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn(CastRays {
					results: HashMap::from([(
						CastRayFor::Beam,
						RayCastResult {
							toi: toi!(11000.),
							..default()
						},
					)]),
				})
				.id();
			let skill_transform = app
				.world_mut()
				.spawn((
					SkillTransformOf(entity),
					PhysicsModel::Beam {
						half_y: Units::from(0.5),
						radius: Units::from(2.),
					},
				))
				.id();

			app.update();

			assert_eq!(
				(
					Some(&Transform {
						translation: Vec3::ZERO.with_z(-5500.),
						..default()
					}),
					Some(&PhysicsModel::Beam {
						half_y: Units::from(5500.),
						radius: Units::from(2.),
					}),
				),
				(
					app.world().entity(skill_transform).get::<Transform>(),
					app.world().entity(skill_transform).get::<PhysicsModel>(),
				)
			);
			Ok(())
		}

		#[test]
		fn do_not_update_collider_when_beam_length_did_not_change() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn(CastRays {
					results: HashMap::from([(
						CastRayFor::Beam,
						RayCastResult {
							toi: toi!(11000.),
							..default()
						},
					)]),
				})
				.id();
			let skill_transform = app
				.world_mut()
				.spawn((
					SkillTransformOf(entity),
					PhysicsModel::Beam {
						half_y: Units::from(0.5),
						radius: Units::from(2.),
					},
				))
				.id();

			app.update();
			app.update();

			assert_eq!(
				Some(&IsChanged::FALSE),
				app.world()
					.entity(skill_transform)
					.get::<IsChanged<PhysicsModel>>(),
			);
			Ok(())
		}
	}
}
