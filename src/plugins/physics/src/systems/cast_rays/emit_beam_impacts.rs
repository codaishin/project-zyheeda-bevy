use crate::{
	components::{
		cast_rays::{CastRayFor, CastRays, RayCastResult},
		collider::ColliderOf,
		impact_able::ImpactAble,
	},
	events::impact_event::ImpactEvent,
};
use bevy::prelude::*;
use common::prelude::*;
use std::collections::HashMap;

impl CastRays {
	pub(crate) fn emit_beam_impacts(
		cast_rays: Query<&Self>,
		mut commands: ZyheedaCommands,
		can_be_impacted: Query<(), With<ImpactAble>>,
		child_colliders: Query<&ColliderOf>,
	) {
		for CastRays { results } in cast_rays {
			let Some((hit, position)) = Self::get_beam_impact(results) else {
				continue;
			};

			let entity = Self::get_root(hit, child_colliders);

			if !can_be_impacted.contains(entity) {
				continue;
			}

			commands.trigger_observers_for(ImpactEvent { entity, position });
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
	use crate::components::cast_rays::RayCasterArgs;
	use std::collections::HashMap;
	use testing::SingleThreadedApp;

	#[derive(Resource, Debug, PartialEq, Default)]
	struct _Record(Vec<ImpactEvent>);

	fn record_impacts(impact: On<ImpactEvent>, mut record: ResMut<_Record>) {
		record.0.push(*impact.event());
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<_Record>();
		app.add_systems(Update, CastRays::emit_beam_impacts);
		app.add_observer(record_impacts);

		app
	}

	#[test]
	fn emit_impact() {
		let mut app = setup();
		let entity = app.world_mut().spawn(ImpactAble).id();
		app.world_mut().spawn(CastRays {
			results: HashMap::from([(
				CastRayFor::Beam,
				RayCastResult {
					hit: Some(entity),
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
			&_Record(vec![ImpactEvent {
				entity,
				position: vec_not_nan!(1., 13., 3.),
			}]),
			app.world().resource::<_Record>(),
		);
	}

	#[test]
	fn emit_impact_for_root() {
		let mut app = setup();
		let entity = app.world_mut().spawn(ImpactAble).id();
		let hit = app.world_mut().spawn(ColliderOf(entity)).id();
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
			&_Record(vec![ImpactEvent {
				entity,
				position: vec_not_nan!(1., 13., 3.),
			}]),
			app.world().resource::<_Record>(),
		);
	}

	#[test]
	fn ignore_non_impact_able() {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();
		app.world_mut().spawn(CastRays {
			results: HashMap::from([(
				CastRayFor::Beam,
				RayCastResult {
					hit: Some(entity),
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

		assert_eq!(&_Record(vec![]), app.world().resource::<_Record>(),);
	}
}
