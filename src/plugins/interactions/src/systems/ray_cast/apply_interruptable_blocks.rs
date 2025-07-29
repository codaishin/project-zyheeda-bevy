use super::execute_ray_caster::RayCastResult;
use crate::components::blockable::Blockable;
use bevy::prelude::*;
use common::{
	components::{
		collider_relationship::ColliderOfInteractionTarget,
		is_blocker::{Blocker, IsBlocker},
	},
	traits::{cast_ray::TimeOfImpact, handles_interactions::BlockableType},
};
use std::collections::{HashMap, HashSet};

pub(crate) fn apply_interruptable_ray_blocks(
	In(mut ray_casts): In<HashMap<Entity, RayCastResult>>,
	mut interruptable_rays: Query<(Entity, &Blockable)>,
	blockers: Query<&IsBlocker>,
	colliders: Query<&ColliderOfInteractionTarget>,
) -> HashMap<Entity, RayCastResult> {
	for (entity, blockable) in &mut interruptable_rays {
		if blockable.blockable_type != BlockableType::Beam {
			continue;
		}

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
			if is_interrupted(&blockable.blockers, hit) {
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

fn is_interrupted(interrupters: &HashSet<Blocker>, hit: Option<&IsBlocker>) -> bool {
	let Some(IsBlocker(blockers)) = hit else {
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
	use common::{
		components::collider_relationship::ColliderOfInteractionTarget,
		traits::{cast_ray::TimeOfImpact, handles_interactions::BlockableDefinition},
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
			.spawn(IsBlocker::from([Blocker::Physical]))
			.id();
		let far = app.world_mut().spawn_empty().id();
		let interruptable = app
			.world_mut()
			.spawn(Blockable::new(BlockableType::Beam, [Blocker::Physical]))
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
			.spawn(IsBlocker::from([Blocker::Physical]))
			.id();
		let blocker = app
			.world_mut()
			.spawn(ColliderOfInteractionTarget::from_raw(root))
			.id();
		let far = app.world_mut().spawn_empty().id();
		let interruptable = app
			.world_mut()
			.spawn(Blockable::new(BlockableType::Beam, [Blocker::Physical]))
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
		let mut app = setup();
		let close = app.world_mut().spawn_empty().id();
		let blocker = app
			.world_mut()
			.spawn(IsBlocker::from([Blocker::Physical]))
			.id();
		let far = app.world_mut().spawn_empty().id();
		let interruptable = app
			.world_mut()
			.spawn(Blockable::new(BlockableType::Beam, []))
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
			.spawn(IsBlocker::from([Blocker::Physical]))
			.id();
		let far = app.world_mut().spawn_empty().id();
		let interruptable = app
			.world_mut()
			.spawn(Blockable::new(BlockableType::Beam, [Blocker::Force]))
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
	fn do_nothing_if_it_is_no_beam() -> Result<(), RunSystemError> {
		let mut app = setup();
		let close = app.world_mut().spawn_empty().id();
		let blocker = app
			.world_mut()
			.spawn(IsBlocker::from([Blocker::Physical]))
			.id();
		let far = app.world_mut().spawn_empty().id();
		let interruptable = app
			.world_mut()
			.spawn(Blockable::new(BlockableType::Fragile, [Blocker::Physical]))
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
