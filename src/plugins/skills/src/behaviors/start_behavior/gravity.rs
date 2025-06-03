use crate::behaviors::{SkillCaster, SkillTarget};
use bevy::ecs::system::EntityCommands;
use common::{
	effects::{EffectApplies, gravity::Gravity},
	tools::UnitsPerSecond,
	traits::{handles_effect::HandlesEffect, handles_skill_behaviors::Spawner},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct StartGravity {
	strength: UnitsPerSecond,
	effect_applies: EffectApplies,
}

impl StartGravity {
	pub fn apply<TEffects>(
		&self,
		entity: &mut EntityCommands,
		_: &SkillCaster,
		_: Spawner,
		_: &SkillTarget,
	) where
		TEffects: HandlesEffect<Gravity>,
	{
		entity.try_insert(TEffects::effect(Gravity {
			strength: self.strength,
			effect_applies: self.effect_applies,
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
		test_tools::utils::SingleThreadedApp,
		traits::clamp_zero_positive::ClampZeroPositive,
	};

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

	fn gravity(
		In((pull, effect_applies)): In<(UnitsPerSecond, EffectApplies)>,
		mut commands: Commands,
	) -> Entity {
		let mut entity = commands.spawn_empty();
		StartGravity {
			strength: pull,
			effect_applies,
		}
		.apply::<_HandlesEffects>(
			&mut entity,
			&SkillCaster::from(Entity::from_raw(42)),
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
			.run_system_once_with(gravity, (UnitsPerSecond::new(42.), EffectApplies::Once))?;

		assert_eq!(
			Some(&_Effect(Gravity {
				strength: UnitsPerSecond::new(42.),
				effect_applies: EffectApplies::Once,
			})),
			app.world().entity(entity).get::<_Effect>()
		);
		Ok(())
	}
}
