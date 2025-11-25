use crate::components::{
	movement_config::MovementConfig,
	player::{PLAYER_RUN, Player},
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::traits::{
	accessors::get::{GetChangedContext, GetContext, GetContextMut},
	animation::{
		ActiveAnimationsMut,
		AnimationKey,
		AnimationPriority,
		Animations,
		AnimationsUnprepared,
	},
	handles_movement::{CurrentMovement, Movement},
};
use std::collections::HashSet;

impl Player {
	pub(crate) fn animate_movement<TMovement, TAnimations>(
		movement: StaticSystemParam<TMovement>,
		mut animations: StaticSystemParam<TAnimations>,
		players: Query<(Entity, &MovementConfig), With<Self>>,
	) -> Result<(), Vec<AnimationsUnprepared>>
	where
		TMovement: for<'c> GetContext<Movement, TContext<'c>: CurrentMovement>,
		TAnimations: for<'c> GetContextMut<Animations, TContext<'c>: ActiveAnimationsMut>,
	{
		let animate_movement = |(entity, config)| {
			let key = Movement { entity };
			let movement = TMovement::get_changed_context(&movement, key)?;

			let key = Animations { entity };
			let mut animations = TAnimations::get_context_mut(&mut animations, key)?;

			let movement_animations = match animations.active_animations_mut(Move) {
				Err(error) => return Some(error),
				Ok(movement_animations) => movement_animations,
			};

			match movement.current_movement() {
				Some(_) => Self::start_run_or_walk_animation(movement_animations, config),
				None => Self::stop_move_animations(movement_animations),
			}

			None
		};

		let errors = players
			.iter()
			.filter_map(animate_movement)
			.collect::<Vec<_>>();

		if !errors.is_empty() {
			return Err(errors);
		}

		Ok(())
	}

	fn start_run_or_walk_animation(
		movement_animations: &mut HashSet<AnimationKey>,
		config: &MovementConfig,
	) {
		let walk_or_run = if config == &*PLAYER_RUN {
			AnimationKey::Run
		} else {
			AnimationKey::Walk
		};

		*movement_animations = HashSet::from([walk_or_run]);
	}

	fn stop_move_animations(movement_animations: &mut HashSet<AnimationKey>) {
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
	use super::*;
	use crate::components::{movement_config::MovementConfig, player::PLAYER_WALK};
	use common::traits::{
		animation::{ActiveAnimations, AnimationKey, AnimationPriority, AnimationsUnprepared},
		handles_movement::MovementTarget,
	};
	use std::{collections::HashMap, sync::LazyLock};
	use test_case::test_case;
	use testing::SingleThreadedApp;

	#[derive(Component)]
	pub(crate) struct _Movement(pub(crate) Option<MovementTarget>);

	impl CurrentMovement for _Movement {
		fn current_movement(&self) -> Option<MovementTarget> {
			self.0
		}
	}

	static EMPTY: LazyLock<HashSet<AnimationKey>> = LazyLock::new(HashSet::default);

	#[derive(Component, Debug, PartialEq)]
	pub(crate) enum _Animations {
		Unprepared(Entity),
		Prepared(HashMap<AnimationPriority, HashSet<AnimationKey>>),
	}

	impl Default for _Animations {
		fn default() -> Self {
			Self::Prepared(HashMap::default())
		}
	}

	impl ActiveAnimations for _Animations {
		fn active_animations<TLayer>(
			&self,
			layer: TLayer,
		) -> Result<&HashSet<AnimationKey>, AnimationsUnprepared>
		where
			TLayer: Into<AnimationPriority>,
		{
			match self {
				_Animations::Unprepared(entity) => Err(AnimationsUnprepared { entity: *entity }),
				_Animations::Prepared(hash_map) => {
					Ok(hash_map.get(&layer.into()).unwrap_or(&*EMPTY))
				}
			}
		}
	}

	impl ActiveAnimationsMut for _Animations {
		fn active_animations_mut<TLayer>(
			&mut self,
			layer: TLayer,
		) -> Result<&mut HashSet<AnimationKey>, AnimationsUnprepared>
		where
			TLayer: Into<AnimationPriority>,
		{
			match self {
				_Animations::Unprepared(entity) => Err(AnimationsUnprepared { entity: *entity }),
				_Animations::Prepared(hash_map) => Ok(hash_map.entry(layer.into()).or_default()),
			}
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	pub(crate) struct _AnimationErrors(pub(crate) Vec<AnimationsUnprepared>);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			Player::animate_movement::<Query<Ref<_Movement>>, Query<Mut<_Animations>>>.pipe(
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

	#[test_case(PLAYER_RUN.clone(), AnimationKey::Run; "run")]
	#[test_case(PLAYER_WALK.clone(), AnimationKey::Walk; "walk")]
	fn start_animation(config: MovementConfig, animation: AnimationKey) {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Player,
				_Movement(Some(MovementTarget::Dir(Dir3::X))),
				config,
				_Animations::default(),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Animations::Prepared(HashMap::from([(
				Move.into(),
				HashSet::from([animation])
			)]))),
			app.world().entity(entity).get::<_Animations>(),
		);
	}

	#[test_case(PLAYER_RUN.clone(), AnimationKey::Run; "run")]
	#[test_case(PLAYER_WALK.clone(), AnimationKey::Walk; "walk")]
	fn override_all_movement_animations(config: MovementConfig, animation: AnimationKey) {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Player,
				_Movement(Some(MovementTarget::Dir(Dir3::X))),
				config,
				_Animations::Prepared(HashMap::from([(
					Move.into(),
					HashSet::from([AnimationKey::Idle]),
				)])),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Animations::Prepared(HashMap::from([(
				Move.into(),
				HashSet::from([animation])
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
				_Movement(Some(MovementTarget::Dir(Dir3::X))),
				PLAYER_RUN.clone(),
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
				_Movement(Some(MovementTarget::Dir(Dir3::X))),
				PLAYER_RUN.clone(),
				_Animations::Prepared(HashMap::from([(
					Move.into(),
					HashSet::from([AnimationKey::Idle]),
				)])),
			))
			.id();

		app.update();
		app.world_mut().entity_mut(entity).insert(_Movement(None));
		app.update();

		assert_eq!(
			Some(&_Animations::Prepared(HashMap::from([(
				Move.into(),
				HashSet::from([]),
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
				_Movement(Some(MovementTarget::Dir(Dir3::X))),
				PLAYER_RUN.clone(),
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
				Player,
				_Movement(Some(MovementTarget::Dir(Dir3::X))),
				PLAYER_RUN.clone(),
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
