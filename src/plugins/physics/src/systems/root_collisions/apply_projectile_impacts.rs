use crate::{
	components::{
		collision_domains::Physical,
		impacted::{ImpactStrength, Impacted},
		projectile::Projectile,
	},
	resources::root_collisions::RootCollisions,
};
use bevy::prelude::*;
use common::tools::vec_not_nan::VecNotNan;
use zyheeda_core::prelude::*;

impl RootCollisions<Physical> {
	pub(crate) fn apply_projectile_impacts(
		root_collisions: Res<Self>,
		impacted: Query<(Entity, &mut Impacted)>,
		impacting: Query<(&GlobalTransform, &Projectile)>,
	) {
		for (entity, mut impacted) in impacted {
			for entity in root_collisions.ongoing(&entity) {
				let Ok((impact, Projectile { leading_edge })) = impacting.get(*entity) else {
					continue;
				};

				let impact = impact.translation() + impact.forward() * **leading_edge;

				let Ok(impact) = VecNotNan::try_from(impact) else {
					continue;
				};

				impacted
					.impact_points
					.insert(impact, new_f32!(ImpactStrength(1.)));
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{impacted::Impacted, projectile::Projectile};
	use common::{tools::Units, vec_not_nan};
	use std::collections::HashMap;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<RootCollisions<Physical>>();
		app.add_systems(Update, RootCollisions::<Physical>::apply_projectile_impacts);

		app
	}

	#[test]
	fn set_impact_with_fragile_translation() {
		let mut app = setup();
		let a = app.world_mut().spawn(Impacted::default()).id();
		let b = app
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
			.update(a, [b]);

		app.update();

		assert_eq!(
			Some(&Impacted {
				impact_points: HashMap::from([(
					vec_not_nan!(1.5, 2., 3.),
					new_f32!(ImpactStrength(1.))
				)])
			}),
			app.world().entity(a).get::<Impacted>(),
		);
	}
}
