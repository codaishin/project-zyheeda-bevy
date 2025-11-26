use crate::components::animate_idle::AnimateIdle;
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{GetContextMut, TryApplyOn},
		animation::{ActiveAnimationsMut, AnimationKey, AnimationPriority, Animations},
	},
	zyheeda_commands::ZyheedaCommands,
};
use std::collections::HashSet;

impl AnimateIdle {
	pub(crate) fn execute<TAnimations>(
		mut commands: ZyheedaCommands,
		mut animations: StaticSystemParam<TAnimations>,
		idlers: Query<Entity, With<Self>>,
	) where
		TAnimations: for<'c> GetContextMut<Animations, TContext<'c>: ActiveAnimationsMut>,
	{
		for entity in &idlers {
			let key = Animations { entity };
			let Some(mut animations) = TAnimations::get_context_mut(&mut animations, key) else {
				continue;
			};
			let Ok(movement_animations) = animations.active_animations_mut(Idle) else {
				continue;
			};

			*movement_animations = HashSet::from([AnimationKey::Idle]);
			commands.try_apply_on(&entity, |mut e| {
				e.try_remove::<Self>();
			});
		}
	}
}

struct Idle;

impl From<Idle> for AnimationPriority {
	fn from(_: Idle) -> Self {
		Self::Low
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::systems::player::animate_movement::tests::_Animations;
	use common::traits::animation::AnimationKey;
	use std::collections::{HashMap, HashSet};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, AnimateIdle::execute::<Query<&mut _Animations>>);

		app
	}

	#[test]
	fn animate_idle() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((AnimateIdle, _Animations::default()))
			.id();

		app.update();

		assert_eq!(
			Some(&_Animations::Prepared(HashMap::from([(
				Idle.into(),
				HashSet::from([AnimationKey::Idle])
			)]))),
			app.world().entity(entity).get::<_Animations>()
		);
	}

	#[test]
	fn override_other_idle_animations() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				AnimateIdle,
				_Animations::Prepared(HashMap::from([(
					Idle.into(),
					HashSet::from([AnimationKey::Walk]),
				)])),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Animations::Prepared(HashMap::from([(
				Idle.into(),
				HashSet::from([AnimationKey::Idle])
			)]))),
			app.world().entity(entity).get::<_Animations>()
		);
	}

	#[test]
	fn do_nothing_if_animate_idle_missing() {
		let mut app = setup();
		let entity = app.world_mut().spawn(_Animations::default()).id();

		app.update();

		assert_eq!(
			Some(&_Animations::default()),
			app.world().entity(entity).get::<_Animations>()
		);
	}

	#[test]
	fn remove_animate_idle_component() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((AnimateIdle, _Animations::default()))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<AnimateIdle>());
	}

	#[test]
	fn do_not_remove_animate_idle_component_when_animations_for_entity_are_missing() {
		let mut app = setup();
		let entity = app.world_mut().spawn(AnimateIdle).id();

		app.update();

		assert_eq!(
			Some(&AnimateIdle),
			app.world().entity(entity).get::<AnimateIdle>(),
		);
	}

	#[test]
	fn do_not_remove_animate_idle_component_when_animations_unprepared() {
		let mut app = setup();
		let entity = app.world_mut().spawn(AnimateIdle).id();
		app.world_mut()
			.entity_mut(entity)
			.insert(_Animations::Unprepared(entity));

		app.update();

		assert_eq!(
			Some(&AnimateIdle),
			app.world().entity(entity).get::<AnimateIdle>(),
		);
	}
}
