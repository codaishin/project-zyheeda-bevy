use crate::behaviors::{SkillCaster, SkillSpawner, Target};
use bevy::ecs::system::EntityCommands;
use common::tools::UnitsPerSecond;
use interactions::components::gravity::Gravity;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct StartGravity {
	strength: UnitsPerSecond,
}

impl StartGravity {
	pub fn apply(
		&self,
		entity: &mut EntityCommands,
		_: &SkillCaster,
		_: &SkillSpawner,
		_: &Target,
	) {
		entity.try_insert(Gravity {
			strength: self.strength,
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::system::RunSystemOnce,
		prelude::{Commands, Entity, In},
	};
	use common::{
		test_tools::utils::SingleThreadedApp,
		traits::clamp_zero_positive::ClampZeroPositive,
	};

	fn gravity(In(pull): In<UnitsPerSecond>, mut commands: Commands) -> Entity {
		let mut entity = commands.spawn_empty();
		StartGravity { strength: pull }.apply(
			&mut entity,
			&SkillCaster::from(Entity::from_raw(42)),
			&SkillSpawner::from(Entity::from_raw(43)),
			&Target::default(),
		);
		entity.id()
	}

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn spawn_gravity_well() {
		let mut app = setup();

		let entity = app
			.world_mut()
			.run_system_once_with(UnitsPerSecond::new(83.), gravity);

		assert_eq!(
			Some(&Gravity {
				strength: UnitsPerSecond::new(83.)
			}),
			app.world().entity(entity).get::<Gravity>()
		);
	}
}
