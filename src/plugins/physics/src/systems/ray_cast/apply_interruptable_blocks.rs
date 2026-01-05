use super::execute_ray_caster::RayCastResult;
use crate::components::{
	blockable::Blockable,
	blocker_types::BlockerTypes,
	interaction_target::ColliderOfInteractionTarget,
};
use bevy::prelude::*;
use common::traits::handles_physics::{PhysicalObject, TimeOfImpact, colliders::Blocker};
use std::collections::{HashMap, HashSet};

pub(crate) fn apply_interruptable_ray_blocks(
	In(mut ray_casts): In<HashMap<Entity, RayCastResult>>,
	mut interruptable_rays: Query<(Entity, &Blockable)>,
	blockers: Query<&BlockerTypes>,
	colliders: Query<&ColliderOfInteractionTarget>,
) -> HashMap<Entity, RayCastResult> {
	for (entity, Blockable(beam)) in &mut interruptable_rays {
		let PhysicalObject::Beam { blocked_by, .. } = beam else {
			continue;
		};

		let Some(ray_cast) = ray_casts.get_mut(&entity) else {
			continue;
		};

		let mut interrupt = None;
		let no_or_first_interruption = |(hit, toi): &(Entity, TimeOfImpact)| {
			if interrupt.is_some() {
				return false;
			}

			let hit = colliders
				.get(*hit)
				.map(|ColliderOfInteractionTarget(target)| *target)
				.unwrap_or(*hit);
			let hit = blockers.get(hit).ok();
			if is_interrupted(blocked_by, hit) {
				interrupt = Some(*toi);
			}
			true
		};

		let info = &mut ray_cast.info;
		info.hits = info
			.hits
			.iter()
			.cloned()
			.take_while(no_or_first_interruption)
			.collect();

		let Some(toi) = interrupt else {
			continue;
		};
		info.max_toi = toi;
	}

	ray_casts
}

fn is_interrupted(interrupters: &HashSet<Blocker>, hit: Option<&BlockerTypes>) -> bool {
	let Some(BlockerTypes(blockers)) = hit else {
		return false;
	};

	is_effected(interrupters, blockers)
}

fn is_effected(interrupters: &HashSet<Blocker>, blockers: &HashSet<Blocker>) -> bool {
	interrupters.intersection(blockers).next().is_some()
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::events::RayCastInfo;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::toi;

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn remove_blocked_hits_from_ray_cast_result() -> Result<(), RunSystemError> {
		let mut app = setup();
		let close = app.world_mut().spawn_empty().id();
		let blocker = app
			.world_mut()
			.spawn(BlockerTypes::from([Blocker::Physical]))
			.id();
		let far = app.world_mut().spawn_empty().id();
		let interruptable = app
			.world_mut()
			.spawn(Blockable(PhysicalObject::Beam {
				range: default(),
				blocked_by: [Blocker::Physical].into(),
			}))
			.id();
		let ray_casts = HashMap::from([(
			interruptable,
			RayCastResult {
				info: RayCastInfo {
					hits: vec![(close, toi!(1.)), (blocker, toi!(2.)), (far, toi!(3.))],
					max_toi: toi!(99.),
					..default()
				},
			},
		)]);

		let ray_casts = app
			.world_mut()
			.run_system_once_with(apply_interruptable_ray_blocks, ray_casts)?;

		assert_eq!(
			HashMap::from([(
				interruptable,
				RayCastResult {
					info: RayCastInfo {
						hits: vec![(close, toi!(1.)), (blocker, toi!(2.))],
						max_toi: toi!(2.),
						..default()
					},
				}
			)]),
			ray_casts
		);
		Ok(())
	}

	#[test]
	fn remove_blocked_hits_from_ray_cast_result_when_using_collider_root()
	-> Result<(), RunSystemError> {
		let mut app = setup();
		let close = app.world_mut().spawn_empty().id();
		let root = app
			.world_mut()
			.spawn(BlockerTypes::from([Blocker::Physical]))
			.id();
		let blocker = app
			.world_mut()
			.spawn(ColliderOfInteractionTarget(root))
			.id();
		let far = app.world_mut().spawn_empty().id();
		let interruptable = app
			.world_mut()
			.spawn(Blockable(PhysicalObject::Beam {
				range: default(),
				blocked_by: [Blocker::Physical].into(),
			}))
			.id();
		let ray_casts = HashMap::from([(
			interruptable,
			RayCastResult {
				info: RayCastInfo {
					hits: vec![(close, toi!(1.)), (blocker, toi!(2.)), (far, toi!(3.))],
					max_toi: toi!(99.),
					..default()
				},
			},
		)]);

		let ray_casts = app
			.world_mut()
			.run_system_once_with(apply_interruptable_ray_blocks, ray_casts)?;

		assert_eq!(
			HashMap::from([(
				interruptable,
				RayCastResult {
					info: RayCastInfo {
						hits: vec![(close, toi!(1.)), (blocker, toi!(2.))],
						max_toi: toi!(2.),
						..default()
					},
				}
			)]),
			ray_casts
		);
		Ok(())
	}

	#[test]
	fn do_nothing_if_not_blocked_by_anything() -> Result<(), RunSystemError> {
		let mut app = setup();
		let close = app.world_mut().spawn_empty().id();
		let blocker = app
			.world_mut()
			.spawn(BlockerTypes::from([Blocker::Physical]))
			.id();
		let far = app.world_mut().spawn_empty().id();
		let interruptable = app
			.world_mut()
			.spawn(Blockable(PhysicalObject::Beam {
				range: default(),
				blocked_by: default(),
			}))
			.id();
		let ray_casts = HashMap::from([(
			interruptable,
			RayCastResult {
				info: RayCastInfo {
					hits: vec![(close, toi!(1.)), (blocker, toi!(2.)), (far, toi!(3.))],
					max_toi: toi!(99.),
					..default()
				},
			},
		)]);

		let ray_casts = app
			.world_mut()
			.run_system_once_with(apply_interruptable_ray_blocks, ray_casts)?;

		assert_eq!(
			HashMap::from([(
				interruptable,
				RayCastResult {
					info: RayCastInfo {
						hits: vec![(close, toi!(1.)), (blocker, toi!(2.)), (far, toi!(3.)),],
						max_toi: toi!(99.),
						..default()
					},
				}
			)]),
			ray_casts
		);
		Ok(())
	}

	#[test]
	fn do_nothing_if_blockers_do_not_match() -> Result<(), RunSystemError> {
		let mut app = setup();
		let close = app.world_mut().spawn_empty().id();
		let blocker = app
			.world_mut()
			.spawn(BlockerTypes::from([Blocker::Physical]))
			.id();
		let far = app.world_mut().spawn_empty().id();
		let interruptable = app
			.world_mut()
			.spawn(Blockable(PhysicalObject::Beam {
				range: default(),
				blocked_by: [Blocker::Force].into(),
			}))
			.id();
		let ray_casts = HashMap::from([(
			interruptable,
			RayCastResult {
				info: RayCastInfo {
					hits: vec![(close, toi!(1.)), (blocker, toi!(2.)), (far, toi!(3.))],
					max_toi: toi!(99.),
					..default()
				},
			},
		)]);

		let ray_casts = app
			.world_mut()
			.run_system_once_with(apply_interruptable_ray_blocks, ray_casts)?;

		assert_eq!(
			HashMap::from([(
				interruptable,
				RayCastResult {
					info: RayCastInfo {
						hits: vec![(close, toi!(1.)), (blocker, toi!(2.)), (far, toi!(3.)),],
						max_toi: toi!(99.),
						..default()
					},
				}
			)]),
			ray_casts
		);
		Ok(())
	}

	#[test]
	fn do_nothing_if_it_is_no_beam() -> Result<(), RunSystemError> {
		let mut app = setup();
		let close = app.world_mut().spawn_empty().id();
		let blocker = app
			.world_mut()
			.spawn(BlockerTypes::from([Blocker::Physical]))
			.id();
		let far = app.world_mut().spawn_empty().id();
		let interruptable = app
			.world_mut()
			.spawn(Blockable(PhysicalObject::Fragile {
				destroyed_by: [Blocker::Physical].into(),
			}))
			.id();
		let ray_casts = HashMap::from([(
			interruptable,
			RayCastResult {
				info: RayCastInfo {
					hits: vec![(close, toi!(1.)), (blocker, toi!(2.)), (far, toi!(3.))],
					max_toi: toi!(99.),
					..default()
				},
			},
		)]);

		let ray_casts = app
			.world_mut()
			.run_system_once_with(apply_interruptable_ray_blocks, ray_casts)?;

		assert_eq!(
			HashMap::from([(
				interruptable,
				RayCastResult {
					info: RayCastInfo {
						hits: vec![(close, toi!(1.)), (blocker, toi!(2.)), (far, toi!(3.)),],
						max_toi: toi!(99.),
						..default()
					},
				}
			)]),
			ray_casts
		);
		Ok(())
	}
}
