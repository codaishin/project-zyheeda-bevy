use crate::components::effects::{
	Effects,
	force::ForceEffect,
	gravity::GravityEffect,
	health_damage::HealthDamageEffect,
};
use bevy::prelude::*;
use common::prelude::*;

impl Effects {
	pub(crate) fn insert(
		on_insert: On<Insert, Self>,
		mut commands: ZyheedaCommands,
		effects: Query<&Self>,
	) {
		let entity = on_insert.entity;
		let Ok(Effects(effects)) = effects.get(entity) else {
			return;
		};
		let Some(mut entity) = commands.get_mut(&entity) else {
			return;
		};

		for effect in effects {
			match effect {
				SkillEffect::Force(effect) => entity.try_insert(ForceEffect(*effect)),
				SkillEffect::Gravity(effect) => entity.try_insert(GravityEffect(*effect)),
				SkillEffect::HealthDamage(effect) => entity.try_insert(HealthDamageEffect(*effect)),
			};
		}

		entity.try_remove::<Self>();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
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
			SkillEffect::Force(Force),
			SkillEffect::HealthDamage(HealthDamage(42., EffectApplies::Once)),
			SkillEffect::Gravity(Gravity {
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
