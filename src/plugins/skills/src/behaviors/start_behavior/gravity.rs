use crate::behaviors::{SkillCaster, SkillSpawner, Target};
use bevy::ecs::system::EntityCommands;
use common::{
	effects::{gravity::Gravity, EffectApplies},
	tools::UnitsPerSecond,
	traits::{handles_effect::HandlesEffect, handles_effect_shading::HandlesEffectShadingFor},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct StartGravity {
	strength: UnitsPerSecond,
	effect_applies: EffectApplies,
}

impl StartGravity {
	pub fn apply<TEffects, TShaders>(
		&self,
		entity: &mut EntityCommands,
		_: &SkillCaster,
		_: &SkillSpawner,
		_: &Target,
	) where
		TEffects: HandlesEffect<Gravity>,
		TShaders: HandlesEffectShadingFor<Gravity>,
	{
		let gravity = Gravity {
			strength: self.strength,
			effect_applies: self.effect_applies,
		};
		entity.try_insert((TEffects::effect(gravity), TShaders::effect_shader(gravity)));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{ecs::system::RunSystemOnce, prelude::*};
	use common::{
		test_tools::utils::SingleThreadedApp,
		traits::clamp_zero_positive::ClampZeroPositive,
	};

	struct _HandlesEffects;

	impl HandlesEffect<Gravity> for _HandlesEffects {
		type TTarget = ();

		fn effect(effect: Gravity) -> impl Bundle {
			_Effect(effect)
		}

		fn attribute(_: Self::TTarget) -> impl Bundle {}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Effect(Gravity);

	struct _HandlesShading;

	impl HandlesEffectShadingFor<Gravity> for _HandlesShading {
		fn effect_shader(effect: Gravity) -> impl Bundle {
			_Shader(effect)
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Shader(Gravity);

	fn gravity(
		In((pull, effect_applies)): In<(UnitsPerSecond, EffectApplies)>,
		mut commands: Commands,
	) -> Entity {
		let mut entity = commands.spawn_empty();
		StartGravity {
			strength: pull,
			effect_applies,
		}
		.apply::<_HandlesEffects, _HandlesShading>(
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
	fn spawn_gravity_shader() {
		let mut app = setup();

		let entity = app.world_mut().run_system_once_with(
			(UnitsPerSecond::new(83.), EffectApplies::OncePerTarget),
			gravity,
		);

		assert_eq!(
			Some(&_Shader(Gravity {
				strength: UnitsPerSecond::new(83.),
				effect_applies: EffectApplies::OncePerTarget,
			})),
			app.world().entity(entity).get::<_Shader>()
		);
	}

	#[test]
	fn spawn_gravity_effect() {
		let mut app = setup();

		let entity = app
			.world_mut()
			.run_system_once_with((UnitsPerSecond::new(42.), EffectApplies::Once), gravity);

		assert_eq!(
			Some(&_Effect(Gravity {
				strength: UnitsPerSecond::new(42.),
				effect_applies: EffectApplies::Once,
			})),
			app.world().entity(entity).get::<_Effect>()
		);
	}
}
