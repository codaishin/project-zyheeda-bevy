use crate::{components::enemy::Enemy, systems::player::animate_movement::Move};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::traits::{
	accessors::get::{GetChangedContext, GetContext, GetContextMut},
	handles_animations::{ActiveAnimationsMut, AnimationKey, Animations, AnimationsUnprepared},
	handles_movement::{CurrentMovement, Movement},
};
use std::collections::HashSet;

impl Enemy {
	pub(crate) fn animate_movement<TMovement, TAnimations>(
		movement: StaticSystemParam<TMovement>,
		mut animations: StaticSystemParam<TAnimations>,
		enemies: Query<Entity, With<Self>>,
	) -> Result<(), Vec<AnimationsUnprepared>>
	where
		TMovement: for<'c> GetContext<Movement, TContext<'c>: CurrentMovement>,
		TAnimations: for<'c> GetContextMut<Animations, TContext<'c>: ActiveAnimationsMut>,
	{
		let animate_movement = |entity| {
			let key = Movement { entity };
			let movement = TMovement::get_changed_context(&movement, key)?;

			let key = Animations { entity };
			let mut animations = TAnimations::get_context_mut(&mut animations, key)?;

			let movement_animations = match animations.active_animations_mut(Move) {
				Err(error) => return Some(error),
				Ok(movement_animations) => movement_animations,
			};

			match movement.current_movement() {
				Some(_) => *movement_animations = HashSet::from([AnimationKey::Run]),
				None => movement_animations.clear(),
			};

			None
		};

		let errors = enemies
			.iter()
			.filter_map(animate_movement)
			.collect::<Vec<_>>();

		if !errors.is_empty() {
			return Err(errors);
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::systems::player::animate_movement::tests::{
		_AnimationErrors,
		_Animations,
		_Movement,
	};
	use common::traits::{handles_animations::AnimationKey, handles_movement::MovementTarget};
	use std::collections::{HashMap, HashSet};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			Enemy::animate_movement::<Query<Ref<_Movement>>, Query<Mut<_Animations>>>.pipe(
				|In(errors), mut commands: Commands| {
					let Err(errors) = errors else {
						return;
					};

					commands.insert_resource(_AnimationErrors(errors));
				},
			),
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
				_Movement(Some(MovementTarget::Dir(Dir3::X))),
				_Animations::default(),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Animations::Prepared(HashMap::from([(
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
				_Movement(Some(MovementTarget::Dir(Dir3::X))),
				_Animations::Prepared(HashMap::from([(
					Move.into(),
					HashSet::from([AnimationKey::Walk]),
				)])),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Animations::Prepared(HashMap::from([(
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
				_Movement(Some(MovementTarget::Dir(Dir3::X))),
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
				_Movement(Some(MovementTarget::Dir(Dir3::X))),
				_Animations::Prepared(HashMap::from([(
					Move.into(),
					HashSet::from([AnimationKey::Run]),
				)])),
			))
			.id();

		app.update();
		app.world_mut().entity_mut(entity).insert(_Movement(None));
		app.update();

		assert_eq!(
			Some(&_Animations::Prepared(HashMap::from([(
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
				_Movement(Some(MovementTarget::Dir(Dir3::X))),
				_Animations::default(),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Animations::default()),
			app.world().entity(entity).get::<_Animations>(),
		);
	}

	#[test]
	fn return_error() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Enemy::default(),
				_Movement(Some(MovementTarget::Dir(Dir3::X))),
			))
			.id();
		app.world_mut()
			.entity_mut(entity)
			.insert(_Animations::Unprepared(entity));

		app.update();

		assert_eq!(
			Some(&_AnimationErrors(vec![AnimationsUnprepared { entity }])),
			app.world().get_resource::<_AnimationErrors>(),
		);
	}
}
