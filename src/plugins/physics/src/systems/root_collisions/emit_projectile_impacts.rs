use crate::{
	components::{collision_domains::Physical, impact_able::ImpactAble, projectile::Projectile},
	events::impact_event::ImpactEvent,
	resources::root_collisions::RootCollisions,
};
use bevy::prelude::*;
use common::prelude::*;

impl RootCollisions<Physical> {
	pub(crate) fn emit_projectile_impacts(
		mut commands: ZyheedaCommands,
		root_collisions: Res<Self>,
		can_be_impacted: Query<(), With<ImpactAble>>,
		projectiles: Query<(Entity, &GlobalTransform, &Projectile)>,
	) {
		for (entity, transform, Projectile { leading_edge }) in projectiles {
			let loading_edge = transform.translation() + transform.forward() * **leading_edge;

			let Ok(position) = VecNotNan::try_from(loading_edge) else {
				continue;
			};

			for entity in root_collisions.ongoing(&entity) {
				if !can_be_impacted.contains(*entity) {
					continue;
				}

				commands.trigger_observers_for(ImpactEvent {
					entity: *entity,
					position,
				});
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{tools::Units, vec_not_nan};
	use testing::SingleThreadedApp;

	#[derive(Resource, Debug, PartialEq, Default)]
	struct _Record(Vec<ImpactEvent>);

	fn record_impacts(impact: On<ImpactEvent>, mut record: ResMut<_Record>) {
		record.0.push(*impact.event());
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<_Record>();
		app.init_resource::<RootCollisions<Physical>>();
		app.add_observer(record_impacts);
		app.add_systems(Update, RootCollisions::<Physical>::emit_projectile_impacts);

		app
	}

	#[test]
	fn set_impact_with_projectile_leading_edge() {
		let mut app = setup();
		let impacted = app.world_mut().spawn(ImpactAble).id();
		let projectile = app
			.world_mut()
			.spawn((
				GlobalTransform::from(Transform::from_xyz(1., 2., 3.).looking_to(Dir3::X, Dir3::Y)),
				Projectile {
					leading_edge: Units::from(0.5),
				},
			))
			.id();
		app.world_mut()
			.resource_mut::<RootCollisions<Physical>>()
			.update(projectile, [impacted]);

		app.update();

		assert_eq!(
			&_Record(vec![ImpactEvent {
				entity: impacted,
				position: vec_not_nan!(1.5, 2., 3.),
			}]),
			app.world().resource::<_Record>(),
		);
	}

	#[test]
	fn ignore_non_impact_able() {
		let mut app = setup();
		let impacted = app.world_mut().spawn_empty().id();
		let projectile = app
			.world_mut()
			.spawn((
				GlobalTransform::from(Transform::from_xyz(1., 2., 3.).looking_to(Dir3::X, Dir3::Y)),
				Projectile {
					leading_edge: Units::from(0.5),
				},
			))
			.id();
		app.world_mut()
			.resource_mut::<RootCollisions<Physical>>()
			.update(projectile, [impacted]);

		app.update();

		assert_eq!(&_Record(vec![]), app.world().resource::<_Record>(),);
	}
}
