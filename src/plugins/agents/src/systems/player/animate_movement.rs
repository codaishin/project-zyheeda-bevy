use crate::components::player::Player;
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	errors::Level,
	traits::{
		accessors::get::{GetChangedContext, GetContext, GetContextMut, Logged, View, ViewOf},
		handles_animations::{ActiveAnimationsMut, AnimationKey, AnimationPriority, Animations},
		handles_movement::{CurrentMovement, Movement, MovementTarget, SpeedToggle},
	},
};
use zyheeda_core::prelude::OrderedSet;

impl Player {
	pub(crate) fn animate_movement<TMovement, TAnimations>(
		movement: StaticSystemParam<TMovement>,
		mut animations: StaticSystemParam<TAnimations>,
		players: Query<Entity, With<Self>>,
	) where
		TMovement: for<'c> GetContext<Logged<Movement>, TContext<'c>: CurrentMovement>,
		TAnimations: for<'c> GetContextMut<Logged<Animations>, TContext<'c>: ActiveAnimationsMut>,
	{
		for entity in players {
			let key = Logged::key(Movement { entity }).with_level(Level::Error);
			let Some(movement) = TMovement::get_changed_context(&movement, key) else {
				continue;
			};

			let key = Logged::key(Animations { entity }).with_level(Level::Error);
			let Some(mut animations) = TAnimations::get_context_mut(&mut animations, key) else {
				continue;
			};

			let movement_animations = animations.active_animations_mut(Move);

			match movement.view_of::<Option<MovementTarget>>() {
				Some(_) => Self::start_run_or_walk_animation(movement_animations, movement),
				None => Self::stop_move_animations(movement_animations),
			}
		}
	}

	fn start_run_or_walk_animation(
		movement_animations: &mut OrderedSet<AnimationKey>,
		config: impl View<SpeedToggle>,
	) {
		let walk_or_run = match config.view() {
			SpeedToggle::Left => AnimationKey::Run,
			SpeedToggle::Right => AnimationKey::Walk,
		};

		*movement_animations = OrderedSet::from([walk_or_run]);
	}

	fn stop_move_animations(movement_animations: &mut OrderedSet<AnimationKey>) {
		movement_animations.clear();
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct Move;

impl From<Move> for AnimationPriority {
	fn from(_: Move) -> Self {
		Self::Medium
	}
}

#[cfg(test)]
pub(crate) mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use common::traits::{
		accessors::get::View,
		handles_animations::{ActiveAnimations, AnimationKey, AnimationPriority},
		handles_movement::MovementTarget,
	};
	use std::collections::HashMap;
	use test_case::test_case;
	use testing::SingleThreadedApp;

	#[derive(Component, Default)]
	pub(crate) struct _Movement {
		pub(crate) target: Option<MovementTarget>,
		pub(crate) speed: SpeedToggle,
	}

	impl View<Option<MovementTarget>> for _Movement {
		fn view(&self) -> Option<MovementTarget> {
			self.target
		}
	}

	impl View<SpeedToggle> for _Movement {
		fn view(&self) -> SpeedToggle {
			self.speed
		}
	}

	const EMPTY: &OrderedSet<AnimationKey> = &OrderedSet::EMPTY;

	#[derive(Component, Debug, PartialEq, Default)]
	pub(crate) struct _Animations(pub(crate) HashMap<AnimationPriority, OrderedSet<AnimationKey>>);

	impl ActiveAnimations for _Animations {
		fn active_animations<TLayer>(&self, layer: TLayer) -> &OrderedSet<AnimationKey>
		where
			TLayer: Into<AnimationPriority>,
		{
			self.0.get(&layer.into()).unwrap_or(EMPTY)
		}
	}

	impl ActiveAnimationsMut for _Animations {
		fn active_animations_mut<TLayer>(&mut self, layer: TLayer) -> &mut OrderedSet<AnimationKey>
		where
			TLayer: Into<AnimationPriority>,
		{
			self.0.entry(layer.into()).or_default()
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			Player::animate_movement::<Query<Ref<_Movement>>, Query<Mut<_Animations>>>,
		);

		app
	}

	fn movement(speed: SpeedToggle) -> _Movement {
		_Movement {
			speed,
			target: Some(MovementTarget::Dir(Dir3::NEG_Z)),
		}
	}

	#[test_case(movement(SpeedToggle::Left), AnimationKey::Run; "run")]
	#[test_case(movement(SpeedToggle::Right), AnimationKey::Walk; "walk")]
	fn start_animation(movement: _Movement, animation: AnimationKey) {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((Player, movement, _Animations::default()))
			.id();

		app.update();

		assert_eq!(
			Some(&_Animations(HashMap::from([(
				Move.into(),
				OrderedSet::from([animation])
			)]))),
			app.world().entity(entity).get::<_Animations>(),
		);
	}

	#[test_case(movement(SpeedToggle::Left), AnimationKey::Run; "run")]
	#[test_case(movement(SpeedToggle::Right), AnimationKey::Walk; "walk")]
	fn override_all_movement_animations(movement: _Movement, animation: AnimationKey) {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Player,
				movement,
				_Animations(HashMap::from([(
					Move.into(),
					OrderedSet::from([AnimationKey::Idle]),
				)])),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Animations(HashMap::from([(
				Move.into(),
				OrderedSet::from([animation])
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
				Player,
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
	fn stop_movement_animations() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Player,
				_Movement {
					target: Some(MovementTarget::Dir(Dir3::X)),
					..default()
				},
				_Animations(HashMap::from([(
					Move.into(),
					OrderedSet::from([AnimationKey::Idle]),
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
				OrderedSet::from([]),
			)])),),
			app.world().entity(entity).get::<_Animations>(),
		);
	}

	#[test]
	fn do_nothing_if_player_is_missing() {
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
