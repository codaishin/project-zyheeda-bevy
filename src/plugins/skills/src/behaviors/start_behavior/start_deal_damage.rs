use crate::behaviors::{SkillCaster, SkillSpawner, Target};
use bevy::ecs::system::EntityCommands;
use interactions::components::deals_damage::DealsDamage;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct StartDealingDamage {
	amount: i16,
}

impl StartDealingDamage {
	pub fn apply(
		&self,
		entity: &mut EntityCommands,
		_: &SkillCaster,
		_: &SkillSpawner,
		_: &Target,
	) {
		entity.try_insert(DealsDamage(self.amount));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::system::RunSystemOnce,
		prelude::{Commands, Entity},
	};
	use common::test_tools::utils::SingleThreadedApp;

	fn damage(damage: StartDealingDamage) -> impl Fn(Commands) -> Entity {
		move |mut commands| {
			let mut entity = commands.spawn_empty();
			damage.apply(
				&mut entity,
				&SkillCaster::from(Entity::from_raw(42)),
				&SkillSpawner::from(Entity::from_raw(43)),
				&Target::default(),
			);
			entity.id()
		}
	}

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn insert_deals_damage() {
		let mut app = setup();

		let start_dealing_damage = StartDealingDamage { amount: 42 };
		let entity = app
			.world_mut()
			.run_system_once(damage(start_dealing_damage));

		assert_eq!(
			Some(&DealsDamage(42)),
			app.world().entity(entity).get::<DealsDamage>()
		);
	}
}
