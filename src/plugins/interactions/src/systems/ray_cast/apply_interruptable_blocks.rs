use super::execute_ray_caster::RayCastResult;
use crate::components::is::{InterruptableRay, Is};
use bevy::prelude::*;
use common::{
	blocker::{Blocker, Blockers},
	components::collider_relationship::ColliderOfInteractionTarget,
	traits::cast_ray::TimeOfImpact,
};
use std::collections::{HashMap, HashSet};

pub(crate) fn apply_interruptable_ray_blocks(
	In(mut ray_casts): In<HashMap<Entity, RayCastResult>>,
	mut interruptable_rays: Query<(Entity, &Is<InterruptableRay>)>,
	blockers: Query<&Blockers>,
	colliders: Query<&ColliderOfInteractionTarget>,
) -> HashMap<Entity, RayCastResult> {
	let colliders = &colliders;
	let blockers = &blockers;

	for (entity, Is(interruptable)) in &mut interruptable_rays {
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
				.map(ColliderOfInteractionTarget::target)
				.unwrap_or(*hit);
			let hit = blockers.get(hit).ok();
			if is_interrupted(interruptable, hit) {
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

fn is_interrupted(interruptable: &InterruptableRay, hit: Option<&Blockers>) -> bool {
	let Some(Blockers(blockers)) = hit else {
		return false;
	};

	is_effected(interruptable, blockers)
}

fn is_effected(InterruptableRay(by): &InterruptableRay, blockers: &HashSet<Blocker>) -> bool {
	by.intersection(blockers).count() != 0
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::events::RayCastInfo;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		components::collider_relationship::ColliderOfInteractionTarget,
		traits::cast_ray::TimeOfImpact,
	};

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn remove_blocked_hits_from_ray_cast_result() -> Result<(), RunSystemError> {
		let mut app = setup();
		let close = app.world_mut().spawn_empty().id();
		let blocker = app
			.world_mut()
			.spawn(Blockers::from([Blocker::Physical]))
			.id();
		let far = app.world_mut().spawn_empty().id();
		let interruptable = app
			.world_mut()
			.spawn(Is::<InterruptableRay>::interacting_with([
				Blocker::Physical,
			]))
			.id();
		let ray_casts = HashMap::from([(
			interruptable,
			RayCastResult {
				info: RayCastInfo {
					hits: vec![
						(close, TimeOfImpact(1.)),
						(blocker, TimeOfImpact(2.)),
						(far, TimeOfImpact(3.)),
					],
					max_toi: TimeOfImpact(99.),
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
						hits: vec![(close, TimeOfImpact(1.)), (blocker, TimeOfImpact(2.))],
						max_toi: TimeOfImpact(2.),
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
			.spawn(Blockers::from([Blocker::Physical]))
			.id();
		let blocker = app
			.world_mut()
			.spawn(ColliderOfInteractionTarget::from_raw(root))
			.id();
		let far = app.world_mut().spawn_empty().id();
		let interruptable = app
			.world_mut()
			.spawn(Is::<InterruptableRay>::interacting_with([
				Blocker::Physical,
			]))
			.id();
		let ray_casts = HashMap::from([(
			interruptable,
			RayCastResult {
				info: RayCastInfo {
					hits: vec![
						(close, TimeOfImpact(1.)),
						(blocker, TimeOfImpact(2.)),
						(far, TimeOfImpact(3.)),
					],
					max_toi: TimeOfImpact(99.),
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
						hits: vec![(close, TimeOfImpact(1.)), (blocker, TimeOfImpact(2.))],
						max_toi: TimeOfImpact(2.),
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
		let no_blockers = [];

		let mut app = setup();
		let close = app.world_mut().spawn_empty().id();
		let blocker = app
			.world_mut()
			.spawn(Blockers::from([Blocker::Physical]))
			.id();
		let far = app.world_mut().spawn_empty().id();
		let interruptable = app
			.world_mut()
			.spawn(Is::<InterruptableRay>::interacting_with(no_blockers))
			.id();
		let ray_casts = HashMap::from([(
			interruptable,
			RayCastResult {
				info: RayCastInfo {
					hits: vec![
						(close, TimeOfImpact(1.)),
						(blocker, TimeOfImpact(2.)),
						(far, TimeOfImpact(3.)),
					],
					max_toi: TimeOfImpact(99.),
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
						hits: vec![
							(close, TimeOfImpact(1.)),
							(blocker, TimeOfImpact(2.)),
							(far, TimeOfImpact(3.)),
						],
						max_toi: TimeOfImpact(99.),
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
			.spawn(Blockers::from([Blocker::Physical]))
			.id();
		let far = app.world_mut().spawn_empty().id();
		let interruptable = app
			.world_mut()
			.spawn(Is::<InterruptableRay>::interacting_with([Blocker::Force]))
			.id();
		let ray_casts = HashMap::from([(
			interruptable,
			RayCastResult {
				info: RayCastInfo {
					hits: vec![
						(close, TimeOfImpact(1.)),
						(blocker, TimeOfImpact(2.)),
						(far, TimeOfImpact(3.)),
					],
					max_toi: TimeOfImpact(99.),
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
						hits: vec![
							(close, TimeOfImpact(1.)),
							(blocker, TimeOfImpact(2.)),
							(far, TimeOfImpact(3.)),
						],
						max_toi: TimeOfImpact(99.),
						..default()
					},
				}
			)]),
			ray_casts
		);
		Ok(())
	}
}
