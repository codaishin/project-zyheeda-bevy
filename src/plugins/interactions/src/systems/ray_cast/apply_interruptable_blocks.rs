use std::collections::HashSet;

use crate::components::{
	blocker::{Blocker, Blockers},
	is::{InterruptableRay, Is},
	RayCastResult,
};
use bevy::prelude::{Entity, Mut, Query};
use common::{components::ColliderRoot, traits::cast_ray::TimeOfImpact};

pub(crate) fn apply_interruptable_ray_blocks(
	mut ray_casts: Query<(Mut<RayCastResult>, &Is<InterruptableRay>)>,
	blockers: Query<&Blockers>,
	roots: Query<&ColliderRoot>,
) {
	let roots = &roots;
	let blockers = &blockers;

	for (mut ray_cast, Is(interruptable)) in &mut ray_casts {
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
	use crate::{
		components::{blocker::Blocker, is::Is},
		events::RayCastInfo,
	};
	use bevy::{
		app::{App, Update},
		prelude::default,
	};
	use common::{
		components::ColliderRoot,
		test_tools::utils::SingleThreadedApp,
		traits::cast_ray::TimeOfImpact,
	};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, apply_interruptable_ray_blocks);

		app
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
		let ray_cast = app
			.world_mut()
			.spawn((
				Is::<InterruptableRay>::interacting_with([Blocker::Physical]),
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
			))
			.id();

		app.update();

		let ray_cast = app.world().entity(ray_cast);

		assert_eq!(
			Some(&RayCastResult {
				info: RayCastInfo {
					hits: vec![(close, TimeOfImpact(1.)), (blocker, TimeOfImpact(2.))],
					max_toi: TimeOfImpact(2.),
					..default()
				},
			}),
			ray_cast.get::<RayCastResult>()
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
		let ray_cast = app
			.world_mut()
			.spawn((
				Is::<InterruptableRay>::interacting_with([Blocker::Physical]),
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
			))
			.id();

		app.update();

		let ray_cast = app.world().entity(ray_cast);

		assert_eq!(
			Some(&RayCastResult {
				info: RayCastInfo {
					hits: vec![(close, TimeOfImpact(1.)), (blocker, TimeOfImpact(2.))],
					max_toi: TimeOfImpact(2.),
					..default()
				},
			}),
			ray_cast.get::<RayCastResult>()
		)
	}

	#[test]
	fn remove_blocked_hits_from_ray_cast_result_for_multiple_iterations() {
		let mut app = setup();
		let close = app.world_mut().spawn_empty().id();
		let blocker = app
			.world_mut()
			.spawn(Blockers::new([Blocker::Physical]))
			.id();
		let far = app.world_mut().spawn_empty().id();
		let ray_casts = [
			app.world_mut()
				.spawn((
					Is::<InterruptableRay>::interacting_with([Blocker::Physical]),
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
				))
				.id(),
			app.world_mut()
				.spawn((
					Is::<InterruptableRay>::interacting_with([Blocker::Physical]),
					RayCastResult {
						info: RayCastInfo {
							hits: vec![
								(close, TimeOfImpact(4.)),
								(blocker, TimeOfImpact(7.)),
								(far, TimeOfImpact(9.)),
							],
							max_toi: TimeOfImpact(99.),
							..default()
						},
					},
				))
				.id(),
		];

		app.update();
		app.update();

		let ray_casts = ray_casts.map(|entity| app.world().entity(entity));

		assert_eq!(
			[
				Some(&RayCastResult {
					info: RayCastInfo {
						hits: vec![(close, TimeOfImpact(1.)), (blocker, TimeOfImpact(2.))],
						max_toi: TimeOfImpact(2.),
						..default()
					},
				}),
				Some(&RayCastResult {
					info: RayCastInfo {
						hits: vec![(close, TimeOfImpact(4.)), (blocker, TimeOfImpact(7.))],
						max_toi: TimeOfImpact(7.),
						..default()
					},
				}),
			],
			ray_casts.map(|e| e.get::<RayCastResult>())
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
		let ray_cast = app
			.world_mut()
			.spawn((
				Is::<InterruptableRay>::interacting_with(no_blockers),
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
			))
			.id();

		app.update();

		let ray_cast = app.world().entity(ray_cast);

		assert_eq!(
			Some(&RayCastResult {
				info: RayCastInfo {
					hits: vec![
						(close, TimeOfImpact(1.)),
						(blocker, TimeOfImpact(2.)),
						(far, TimeOfImpact(3.)),
					],
					max_toi: TimeOfImpact(99.),
					..default()
				},
			}),
			ray_cast.get::<RayCastResult>()
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
		let ray_cast = app
			.world_mut()
			.spawn((
				Is::<InterruptableRay>::interacting_with([Blocker::Force]),
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
			))
			.id();

		app.update();

		let ray_cast = app.world().entity(ray_cast);

		assert_eq!(
			Some(&RayCastResult {
				info: RayCastInfo {
					hits: vec![
						(close, TimeOfImpact(1.)),
						(blocker, TimeOfImpact(2.)),
						(far, TimeOfImpact(3.)),
					],
					max_toi: TimeOfImpact(99.),
					..default()
				},
			}),
			ray_cast.get::<RayCastResult>()
		)
	}
}
