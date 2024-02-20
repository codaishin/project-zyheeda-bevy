use crate::components::Beam;
use bevy::{
	ecs::system::{Commands, Query},
	transform::components::GlobalTransform,
};
use bevy_rapier3d::pipeline::QueryFilter;
use common::traits::cast_ray::TimeOfImpact;
use interactions::components::{RayCaster, RayFilter};

pub(crate) fn execute_beam(
	mut commands: Commands,
	void_beams: Query<&Beam>,
	transforms: Query<&GlobalTransform>,
) {
	let origin_and_target = |beam: &Beam| {
		let origin = transforms.get(beam.source).ok()?.translation();
		let target = transforms.get(beam.target).ok()?.translation();
		let filter: RayFilter = QueryFilter::default()
			.exclude_rigid_body(beam.source)
			.try_into()
			.ok()?;
		Some((*beam, origin, target, filter))
	};

	for (beam, origin, target, filter) in void_beams.iter().filter_map(origin_and_target) {
		commands.entity(beam.source).insert(RayCaster {
			origin,
			direction: (target - origin).normalize(),
			solid: true,
			filter,
			max_toi: TimeOfImpact(beam.range),
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		math::Vec3,
	};
	use bevy_rapier3d::pipeline::QueryFilter;
	use common::{test_tools::utils::SingleThreadedApp, traits::cast_ray::TimeOfImpact};
	use interactions::{components::RayCaster, events::RayCastEvent};

	fn setup() -> App {
		let mut app = App::new_single_threaded([Update]);
		app.add_systems(Update, execute_beam);
		app.add_event::<RayCastEvent>();

		app
	}

	#[test]
	fn spawn_ray_caster() {
		let mut app = setup();
		let source = app.world.spawn(GlobalTransform::from_xyz(1., 0., 0.)).id();
		let target = app.world.spawn(GlobalTransform::from_xyz(1., 0., 4.)).id();
		app.world.spawn(Beam {
			source,
			target,
			range: 100.,
		});

		app.update();

		let ray_caster = app.world.entity(source).get::<RayCaster>();

		assert_eq!(
			Some(&RayCaster {
				origin: Vec3::new(1., 0., 0.),
				direction: Vec3::new(0., 0., 1.),
				max_toi: TimeOfImpact(100.),
				solid: true,
				filter: QueryFilter::default()
					.exclude_rigid_body(source)
					.try_into()
					.unwrap(),
			}),
			ray_caster
		);
	}
}
