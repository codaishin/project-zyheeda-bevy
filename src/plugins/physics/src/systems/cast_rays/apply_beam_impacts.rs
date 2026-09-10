use crate::components::{
	cast_rays::{CastRayFor, CastRays, RayCastResult},
	collider::ColliderOf,
	impacted::Impacted,
};
use bevy::prelude::*;
use common::prelude::*;
use std::collections::HashMap;
use zyheeda_core::prelude::*;

impl CastRays {
	pub(crate) fn apply_beam_impacts(
		cast_rays: Query<&Self>,
		mut impacted: Query<&mut Impacted>,
		child_colliders: Query<&ColliderOf>,
	) {
		for CastRays { results } in cast_rays {
			let Some((hit, point)) = Self::get_beam_impact(results) else {
				continue;
			};

			let hit = Self::get_root(hit, child_colliders);

			let Ok(mut impacted) = impacted.get_mut(hit) else {
				continue;
			};

			impacted
				.impact_points
				.insert(point, new_f32!(F32FiniteStrictlyPositive(1.0)));
		}
	}

	fn get_beam_impact(
		results: &HashMap<CastRayFor, RayCastResult>,
	) -> Option<(Entity, VecNotNan<3>)> {
		let result = results.get(&CastRayFor::Beam)?;

		Some((
			result.hit?,
			VecNotNan::try_from(result.args.origin + result.args.direction * *result.toi).ok()?,
		))
	}

	fn get_root(entity: Entity, child_colliders: Query<&ColliderOf>) -> Entity {
		match child_colliders.get(entity) {
			Ok(ColliderOf(root)) => *root,
			Err(_) => entity,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{
		cast_rays::{CastRayFor, RayCastResult, RayCasterArgs},
		collider::ColliderOf,
		impacted::Impacted,
	};
	use std::collections::HashMap;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, CastRays::apply_beam_impacts);

		app
	}

	#[test]
	fn insert_impact() {
		let mut app = setup();
		let hit = app.world_mut().spawn(Impacted::default()).id();
		app.world_mut().spawn(CastRays {
			results: HashMap::from([(
				CastRayFor::Beam,
				RayCastResult {
					hit: Some(hit),
					toi: toi!(11.),
					args: RayCasterArgs {
						origin: Vec3::new(1., 2., 3.),
						direction: Dir3::Y,
						..default()
					},
				},
			)]),
		});

		app.update();

		assert_eq!(
			Some(&Impacted {
				impact_points: HashMap::from([(
					vec_not_nan!(1., 13., 3.),
					new_f32!(F32FiniteStrictlyPositive(1.))
				)])
			}),
			app.world().entity(hit).get::<Impacted>(),
		);
	}

	#[test]
	fn insert_impact_on_root() {
		let mut app = setup();
		let root = app.world_mut().spawn(Impacted::default()).id();
		let hit = app.world_mut().spawn(ColliderOf(root)).id();
		app.world_mut().spawn(CastRays {
			results: HashMap::from([(
				CastRayFor::Beam,
				RayCastResult {
					hit: Some(hit),
					toi: toi!(11.),
					args: RayCasterArgs {
						origin: Vec3::new(1., 2., 3.),
						direction: Dir3::Y,
						..default()
					},
				},
			)]),
		});

		app.update();

		assert_eq!(
			Some(&Impacted {
				impact_points: HashMap::from([(
					vec_not_nan!(1., 13., 3.),
					new_f32!(F32FiniteStrictlyPositive(1.))
				)])
			}),
			app.world().entity(root).get::<Impacted>(),
		);
	}
}
