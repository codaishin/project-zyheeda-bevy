use crate::{
	DestroyAfterDistanceTraveled,
	components::skill_behavior::skill_contact::SkillContact,
};
use bevy::prelude::*;
use common::traits::handles_skill_behaviors::Motion;

impl SkillContact {
	pub(crate) fn update_range(mut contacts: Query<(&mut Self, &DestroyAfterDistanceTraveled)>) {
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
	use common::{
		components::persistent_entity::PersistentEntity,
		tools::{Units, UnitsPerSecond},
		traits::handles_skill_behaviors::{ContactShape, Motion, SkillSpawner},
	};
	use testing::SingleThreadedApp;

	impl SkillContact {
		fn fake_projectile_motion(range: Units) -> Self {
			Self {
				created_from: CreatedFrom::Contact,
				shape: ContactShape::Sphere {
					radius: Units::from(1.),
					hollow_collider: false,
					destroyed_by: default(),
				},
				motion: Motion::Projectile {
					caster: PersistentEntity::default(),
					spawner: SkillSpawner::Neutral,
					speed: UnitsPerSecond::from(1.),
					range,
				},
			}
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, SkillContact::update_range);

		app
	}

	#[test]
	fn take_range_from_traveled_distance() {
		let mut app = setup();
		let contact = app
			.world_mut()
			.spawn((
				SkillContact::fake_projectile_motion(Units::from(100.)),
				WhenTraveled::distance(Units::from(42.)).destroy(),
			))
			.id();

		app.update();

		assert_eq!(
			Some(Units::from(42.)),
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
