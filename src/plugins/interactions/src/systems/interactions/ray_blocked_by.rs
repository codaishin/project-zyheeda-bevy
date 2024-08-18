use crate::components::{BlockedBy, RayCastResult};
use bevy::prelude::{Component, Entity, Mut, Query, With};
use common::{components::ColliderRoot, traits::cast_ray::TimeOfImpact};

pub(crate) fn ray_blocked_by<TBlocker: Component>(
	mut ray_casts: Query<Mut<RayCastResult>, With<BlockedBy<TBlocker>>>,
	blockers: Query<(), With<TBlocker>>,
	roots: Query<&ColliderRoot>,
) {
	let roots = &roots;
	let blockers = &blockers;
	let mut blocked = false;

	let not_blocked = move |(hit, ..): &(Entity, TimeOfImpact)| {
		let was_not_blocked = !blocked;
		let hit = roots.get(*hit).map(|ColliderRoot(r)| *r).unwrap_or(*hit);
		blocked = blockers.get(hit).is_ok();
		was_not_blocked
	};

	for mut ray_cast in &mut ray_casts {
		let info = &mut ray_cast.info;
		info.hits = info.hits.iter().cloned().take_while(not_blocked).collect();

		let Some((.., last_toi)) = info.hits.last() else {
			continue;
		};
		info.max_toi = *last_toi;
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::BlockedBy, events::RayCastInfo};
	use bevy::{
		app::{App, Update},
		prelude::default,
	};
	use common::{
		components::ColliderRoot,
		test_tools::utils::SingleThreadedApp,
		traits::cast_ray::TimeOfImpact,
	};

	#[derive(Component)]
	struct _Blocker;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, ray_blocked_by::<_Blocker>);

		app
	}

	#[test]
	fn remove_blocked_hits_from_ray_cast_result() {
		let mut app = setup();
		let close = app.world_mut().spawn_empty().id();
		let blocker = app.world_mut().spawn(_Blocker).id();
		let far = app.world_mut().spawn_empty().id();
		let ray_cast = app
			.world_mut()
			.spawn((
				BlockedBy::component::<_Blocker>(),
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
		let root = app.world_mut().spawn(_Blocker).id();
		let blocker = app.world_mut().spawn(ColliderRoot(root)).id();
		let far = app.world_mut().spawn_empty().id();
		let ray_cast = app
			.world_mut()
			.spawn((
				BlockedBy::component::<_Blocker>(),
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
		let blocker = app.world_mut().spawn(_Blocker).id();
		let far = app.world_mut().spawn_empty().id();
		let ray_casts = [
			app.world_mut()
				.spawn((
					BlockedBy::component::<_Blocker>(),
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
					BlockedBy::component::<_Blocker>(),
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
		#[derive(Component)]
		struct _NotBlocked;

		let mut app = setup();
		let close = app.world_mut().spawn_empty().id();
		let blocker = app.world_mut().spawn(_Blocker).id();
		let far = app.world_mut().spawn_empty().id();
		let ray_cast = app
			.world_mut()
			.spawn((
				_NotBlocked,
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
