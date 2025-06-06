use crate::{InsertAfterDistanceTraveled, components::skill_behavior::skill_contact::SkillContact};
use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;
use common::traits::handles_skill_behaviors::Motion;

impl SkillContact {
	pub(crate) fn update_range<TDestroy>(
		mut contacts: Query<(&mut Self, &InsertAfterDistanceTraveled<TDestroy, Velocity>)>,
	) where
		TDestroy: Component,
	{
		for (mut contact, range_limiter) in &mut contacts {
			let Motion::Projectile { range, .. } = &mut contact.motion else {
				continue;
			};
			*range = range_limiter.remaining_distance();
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{
		skill_behavior::skill_contact::CreatedFrom,
		when_traveled_insert::WhenTraveled,
	};
	use bevy_rapier3d::prelude::Velocity;
	use common::{
		components::persistent_entity::PersistentEntity,
		test_tools::utils::SingleThreadedApp,
		tools::{Units, UnitsPerSecond},
		traits::{
			clamp_zero_positive::ClampZeroPositive,
			handles_skill_behaviors::{Integrity, Motion, Shape, Spawner},
		},
	};

	#[derive(Component)]
	struct _Destroy;

	impl SkillContact {
		fn fake_projectile_motion(range: Units) -> Self {
			Self {
				created_from: CreatedFrom::Contact,
				shape: Shape::Sphere {
					radius: Units::new(1.),
					hollow_collider: false,
				},
				integrity: Integrity::Solid,
				motion: Motion::Projectile {
					caster: PersistentEntity::default(),
					spawner: Spawner::Center,
					speed: UnitsPerSecond::new(1.),
					range,
				},
			}
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, SkillContact::update_range::<_Destroy>);

		app
	}

	#[test]
	fn take_range_from_traveled_distance() {
		let mut app = setup();
		let contact = app
			.world_mut()
			.spawn((
				SkillContact::fake_projectile_motion(Units::new(100.)),
				WhenTraveled::via::<Velocity>()
					.distance(Units::new(42.))
					.insert::<_Destroy>(),
			))
			.id();

		app.update();

		assert_eq!(
			Some(Units::new(42.)),
			app.world()
				.entity(contact)
				.get::<SkillContact>()
				.and_then(|c| match c.motion {
					Motion::Projectile { range, .. } => Some(range),
					_ => None,
				})
		);
	}
}
