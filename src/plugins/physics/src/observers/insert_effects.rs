use crate::components::effects::{
	Effects,
	force::ForceEffect,
	gravity::GravityEffect,
	health_damage::HealthDamageEffect,
};
use bevy::prelude::*;
use common::{
	traits::{accessors::get::GetMut, handles_skill_physics::Effect},
	zyheeda_commands::ZyheedaCommands,
};

impl Effects {
	pub(crate) fn insert(
		trigger: Trigger<OnInsert, Self>,
		mut commands: ZyheedaCommands,
		effects: Query<&Self>,
	) {
		let entity = trigger.target();
		let Ok(Effects(effects)) = effects.get(entity) else {
			return;
		};
		let Some(mut entity) = commands.get_mut(&entity) else {
			return;
		};

		for effect in effects {
			match effect {
				Effect::Force(effect) => entity.try_insert(ForceEffect(*effect)),
				Effect::Gravity(effect) => entity.try_insert(GravityEffect(*effect)),
				Effect::HealthDamage(effect) => entity.try_insert(HealthDamageEffect(*effect)),
			};
		}

		entity.try_remove::<Self>();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		effects::{EffectApplies, force::Force, gravity::Gravity, health_damage::HealthDamage},
		tools::UnitsPerSecond,
		traits::handles_skill_physics::Effect,
	};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(Effects::insert);

		app
	}

	#[test]
	fn insert_effects() {
		let mut app = setup();

		let entity = app.world_mut().spawn(Effects(vec![
			Effect::Force(Force),
			Effect::HealthDamage(HealthDamage(42., EffectApplies::Once)),
			Effect::Gravity(Gravity {
				strength: UnitsPerSecond::from(11.),
			}),
		]));

		assert_eq!(
			(
				Some(&ForceEffect(Force)),
				Some(&HealthDamageEffect(HealthDamage(42., EffectApplies::Once))),
				Some(&GravityEffect(Gravity {
					strength: UnitsPerSecond::from(11.),
				})),
			),
			(
				entity.get::<ForceEffect>(),
				entity.get::<HealthDamageEffect>(),
				entity.get::<GravityEffect>(),
			)
		);
	}

	#[test]
	fn remove_effects_component() {
		let mut app = setup();

		let entity = app.world_mut().spawn(Effects(vec![]));

		assert_eq!(None, entity.get::<Effects>());
	}
}
