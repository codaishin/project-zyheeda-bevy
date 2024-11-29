use super::execute_ray_caster::RayCastResult;
use crate::components::{
	blockers::Blockers,
	is::{InterruptableRay, Is},
};
use bevy::prelude::{Entity, In, Query};
use common::{blocker::Blocker, components::ColliderRoot, traits::cast_ray::TimeOfImpact};
use std::collections::{HashMap, HashSet};

pub(crate) fn apply_interruptable_ray_blocks(
	In(mut ray_casts): In<HashMap<Entity, RayCastResult>>,
	mut interruptable_rays: Query<(Entity, &Is<InterruptableRay>)>,
	blockers: Query<&Blockers>,
	roots: Query<&ColliderRoot>,
) -> HashMap<Entity, RayCastResult> {
	let roots = &roots;
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

			let hit = roots.get(*hit).map(|ColliderRoot(r)| *r).unwrap_or(*hit);
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
	use bevy::{app::App, ecs::system::RunSystemOnce, prelude::default};
	use common::{components::ColliderRoot, traits::cast_ray::TimeOfImpact};

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn remove_blocked_hits_from_ray_cast_result() {
		let mut app = setup();
		let close = app.world_mut().spawn_empty().id();
		let blocker = app
			.world_mut()
			.spawn(Blockers::new([Blocker::Physical]))
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
			.run_system_once_with(ray_casts, apply_interruptable_ray_blocks);

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
		)
	}

	#[test]
	fn remove_blocked_hits_from_ray_cast_result_when_using_collider_root() {
		let mut app = setup();
		let close = app.world_mut().spawn_empty().id();
		let root = app
			.world_mut()
			.spawn(Blockers::new([Blocker::Physical]))
			.id();
		let blocker = app.world_mut().spawn(ColliderRoot(root)).id();
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
			.run_system_once_with(ray_casts, apply_interruptable_ray_blocks);

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
		)
	}

	#[test]
	fn do_nothing_if_not_blocked_by_anything() {
		let no_blockers = [];

		let mut app = setup();
		let close = app.world_mut().spawn_empty().id();
		let blocker = app
			.world_mut()
			.spawn(Blockers::new([Blocker::Physical]))
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
			.run_system_once_with(ray_casts, apply_interruptable_ray_blocks);

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
		)
	}

	#[test]
	fn do_nothing_if_blockers_do_not_match() {
		let mut app = setup();
		let close = app.world_mut().spawn_empty().id();
		let blocker = app
			.world_mut()
			.spawn(Blockers::new([Blocker::Physical]))
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
			.run_system_once_with(ray_casts, apply_interruptable_ray_blocks);

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
		)
	}
}
