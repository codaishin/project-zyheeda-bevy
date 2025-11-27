use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	tools::action_key::slot::PlayerSlot,
	traits::{
		accessors::get::{GetContextMut, TryApplyOn},
		handles_animations::{ActiveAnimationsMut, AnimationKey, AnimationPriority, Animations},
	},
	zyheeda_commands::ZyheedaCommands,
};

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub enum SkillAnimation {
	Start(PlayerSlot),
	Stop,
}

impl SkillAnimation {
	pub(crate) fn system<TAnimations>(
		mut commands: ZyheedaCommands,
		mut animations: StaticSystemParam<TAnimations>,
		mut players: Query<(Entity, &SkillAnimation)>,
	) where
		TAnimations: for<'c> GetContextMut<Animations, TContext<'c>: ActiveAnimationsMut>,
	{
		for (entity, apply) in &mut players {
			let key = Animations { entity };
			let Some(mut ctx) = TAnimations::get_context_mut(&mut animations, key) else {
				continue;
			};
			let Ok(skill_animations) = ctx.active_animations_mut(Skill) else {
				continue;
			};

			match apply {
				SkillAnimation::Start(slot) => {
					skill_animations.insert(AnimationKey::Skill((*slot).into()));
				}
				SkillAnimation::Stop => {
					skill_animations.clear();
				}
			};
			commands.try_apply_on(&entity, |mut e| {
				e.try_remove::<SkillAnimation>();
			});
		}
	}
}

#[derive(Debug, PartialEq)]
struct Skill;

impl From<Skill> for AnimationPriority {
	fn from(_: Skill) -> Self {
		AnimationPriority::High
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::player::Player,
		systems::player::animate_movement::tests::_Animations,
	};
	use common::tools::action_key::slot::{Side, SlotKey};
	use std::collections::{HashMap, HashSet};
	use test_case::test_case;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, SkillAnimation::system::<Query<Mut<_Animations>>>);

		app
	}

	#[test_case(PlayerSlot::Upper(Side::Left); "top left")]
	#[test_case(PlayerSlot::Upper(Side::Right); "top right")]
	#[test_case(PlayerSlot::Lower(Side::Left); "bottom left")]
	#[test_case(PlayerSlot::Lower(Side::Right); "bottom right")]
	fn play_animation(slot: PlayerSlot) {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((Player, SkillAnimation::Start(slot), _Animations::default()))
			.id();

		app.update();

		assert_eq!(
			Some(&_Animations::Prepared(HashMap::from([(
				Skill.into(),
				HashSet::from([AnimationKey::Skill(slot.into())])
			)]))),
			app.world().entity(entity).get::<_Animations>()
		);
	}

	#[test]
	fn stop_animation() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Player,
				SkillAnimation::Stop,
				_Animations::Prepared(HashMap::from([(
					Skill.into(),
					HashSet::from([AnimationKey::Skill(SlotKey(42))]),
				)])),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Animations::Prepared(HashMap::from([(
				Skill.into(),
				HashSet::default()
			)]))),
			app.world().entity(entity).get::<_Animations>()
		);
	}

	#[test]
	fn remove_skill_animation() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Player,
				SkillAnimation::Start(PlayerSlot::Upper(Side::Left)),
				_Animations::default(),
			))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<SkillAnimation>());
	}
}
