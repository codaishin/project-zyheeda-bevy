use crate::{components::enemy::Enemy, systems::player::animate_movement::Move};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::traits::{
	accessors::get::{GetChangedContext, GetContext, GetContextMut, View},
	handles_animations::{ActiveAnimationsMut, AnimationKey, Animations},
	handles_movement::{Movement, MovementTarget},
};
use std::collections::HashSet;

impl Enemy {
	pub(crate) fn animate_movement<TMovement, TAnimations>(
		movement: StaticSystemParam<TMovement>,
		mut animations: StaticSystemParam<TAnimations>,
		enemies: Query<Entity, With<Self>>,
	) where
		TMovement: for<'c> GetContext<Movement, TContext<'c>: View<Option<MovementTarget>>>,
		TAnimations: for<'c> GetContextMut<Animations, TContext<'c>: ActiveAnimationsMut>,
	{
		for entity in enemies {
			let key = Movement { entity };
			let Some(movement) = TMovement::get_changed_context(&movement, key) else {
				continue;
			};

			let key = Animations { entity };
			let Some(mut animations) = TAnimations::get_context_mut(&mut animations, key) else {
				continue;
			};

			let movement_animations = animations.active_animations_mut(Move);

			match movement.view() {
				Some(_) => *movement_animations = HashSet::from([AnimationKey::Run]),
				None => movement_animations.clear(),
			};
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::systems::player::animate_movement::tests::{_Animations, _Movement};
	use common::traits::{handles_animations::AnimationKey, handles_movement::MovementTarget};
	use std::collections::{HashMap, HashSet};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			Enemy::animate_movement::<Query<Ref<_Movement>>, Query<Mut<_Animations>>>,
		);

		app
	}

	#[test]
	fn start_animation_run() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Enemy::default(),
				_Movement {
					target: Some(MovementTarget::Dir(Dir3::X)),
					..default()
				},
				_Animations::default(),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Animations(HashMap::from([(
				Move.into(),
				HashSet::from([AnimationKey::Run]),
			)]))),
			app.world().entity(entity).get::<_Animations>(),
		);
	}

	#[test]
	fn override_other_movement_animations() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Enemy::default(),
				_Movement {
					target: Some(MovementTarget::Dir(Dir3::X)),
					..default()
				},
				_Animations(HashMap::from([(
					Move.into(),
					HashSet::from([AnimationKey::Walk]),
				)])),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Animations(HashMap::from([(
				Move.into(),
				HashSet::from([AnimationKey::Run]),
			)]))),
			app.world().entity(entity).get::<_Animations>(),
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Enemy::default(),
				_Movement {
					target: Some(MovementTarget::Dir(Dir3::X)),
					..default()
				},
				_Animations::default(),
			))
			.id();

		app.update();
		*app.world_mut()
			.entity_mut(entity)
			.get_mut::<_Animations>()
			.unwrap() = _Animations::default();
		app.update();

		assert_eq!(
			Some(&_Animations::default()),
			app.world().entity(entity).get::<_Animations>(),
		);
	}

	#[test]
	fn stop_animation() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Enemy::default(),
				_Movement {
					target: Some(MovementTarget::Dir(Dir3::X)),
					..default()
				},
				_Animations(HashMap::from([(
					Move.into(),
					HashSet::from([AnimationKey::Run]),
				)])),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(_Movement::default());
		app.update();

		assert_eq!(
			Some(&_Animations(HashMap::from([(
				Move.into(),
				HashSet::from([])
			)]))),
			app.world().entity(entity).get::<_Animations>(),
		);
	}

	#[test]
	fn do_nothing_if_enemy_is_missing() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				_Movement {
					target: Some(MovementTarget::Dir(Dir3::X)),
					..default()
				},
				_Animations::default(),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Animations::default()),
			app.world().entity(entity).get::<_Animations>(),
		);
	}
}
