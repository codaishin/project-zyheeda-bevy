use crate::behaviors::{SkillCaster, SkillTarget};
use bevy::ecs::system::EntityCommands;
use common::{
	effects::gravity::Gravity,
	tools::UnitsPerSecond,
	traits::{handles_effect::HandlesEffect, handles_skill_behaviors::Spawner},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct StartGravity {
	strength: UnitsPerSecond,
}

impl StartGravity {
	pub fn apply<TInteractions>(
		&self,
		entity: &mut EntityCommands,
		_: &SkillCaster,
		_: Spawner,
		_: &SkillTarget,
	) where
		TInteractions: HandlesEffect<Gravity>,
	{
		entity.try_insert(TInteractions::effect(Gravity {
			strength: self.strength,
		}));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::{
		components::persistent_entity::PersistentEntity,
		traits::clamp_zero_positive::ClampZeroPositive,
	};
	use std::sync::LazyLock;
	use testing::SingleThreadedApp;

	struct _HandlesEffects;

	impl HandlesEffect<Gravity> for _HandlesEffects {
		type TTarget = ();
		type TEffectComponent = _Effect;

		fn effect(effect: Gravity) -> _Effect {
			_Effect(effect)
		}

		fn attribute(_: Self::TTarget) -> impl Bundle {}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Effect(Gravity);

	static CASTER: LazyLock<PersistentEntity> = LazyLock::new(PersistentEntity::default);

	fn gravity(In(pull): In<UnitsPerSecond>, mut commands: Commands) -> Entity {
		let mut entity = commands.spawn_empty();
		StartGravity { strength: pull }.apply::<_HandlesEffects>(
			&mut entity,
			&SkillCaster::from(*CASTER),
			Spawner::Center,
			&SkillTarget::default(),
		);
		entity.id()
	}

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn spawn_gravity_effect() -> Result<(), RunSystemError> {
		let mut app = setup();

		let entity = app
			.world_mut()
			.run_system_once_with(gravity, UnitsPerSecond::new(42.))?;

		assert_eq!(
			Some(&_Effect(Gravity {
				strength: UnitsPerSecond::new(42.),
			})),
			app.world().entity(entity).get::<_Effect>()
		);
		Ok(())
	}
}
