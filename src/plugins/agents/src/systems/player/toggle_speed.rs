use crate::components::{
	movement_config::MovementConfig,
	player::{PLAYER_RUN, PLAYER_WALK, Player},
};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	tools::action_key::movement::MovementKey,
	traits::{
		accessors::get::{EntityContextMut, TryApplyOn},
		handles_input::{GetAllInputStates, InputState},
		handles_movement::{Movement, UpdateMovement},
	},
	zyheeda_commands::ZyheedaCommands,
};

impl Player {
	pub(crate) fn toggle_speed<TInput, TMovement>(
		mut commands: ZyheedaCommands,
		mut movement: StaticSystemParam<TMovement>,
		input: StaticSystemParam<TInput>,
		players: Query<(Entity, &MovementConfig), With<Self>>,
	) where
		for<'w, 's> TInput: SystemParam<Item<'w, 's>: GetAllInputStates>,
		for<'c> TMovement: EntityContextMut<Movement, TContext<'c>: UpdateMovement>,
	{
		let just_toggled = input
			.get_all_input_states::<MovementKey>()
			.any(just_toggled);

		if !just_toggled {
			return;
		}

		for (entity, config) in &players {
			let ctx = TMovement::get_entity_context_mut(&mut movement, entity, Movement);
			let Some(mut ctx) = ctx else {
				continue;
			};
			let new_config = if config == &*PLAYER_RUN {
				&*PLAYER_WALK
			} else {
				&*PLAYER_RUN
			};
			ctx.update(new_config.speed, new_config.animation.clone());

			commands.try_apply_on(&entity, move |mut e| {
				e.try_insert(new_config.clone());
			});
		}
	}
}

fn just_toggled((key, state): (MovementKey, InputState)) -> bool {
	key == MovementKey::ToggleWalkRun && state == InputState::just_pressed()
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::player::PLAYER_WALK;
	use common::{
		tools::{
			UnitsPerSecond,
			action_key::{ActionKey, movement::MovementKey},
		},
		traits::{animation::Animation, handles_input::InputState, iteration::IterFinite},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Resource, NestedMocks)]
	struct _Input {
		mock: Mock_Input,
	}

	#[automock]
	impl GetAllInputStates for _Input {
		fn get_all_input_states<TAction>(&self) -> impl Iterator<Item = (TAction, InputState)>
		where
			TAction: Into<ActionKey> + IterFinite + 'static,
		{
			self.mock.get_all_input_states()
		}
	}

	#[derive(Component, NestedMocks)]
	struct _Movement {
		mock: Mock_Movement,
	}

	#[automock]
	impl UpdateMovement for _Movement {
		fn update(&mut self, speed: UnitsPerSecond, animation: Option<Animation>) {
			self.mock.update(speed, animation);
		}
	}

	fn setup(input: _Input) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			Player::toggle_speed::<Res<_Input>, Query<&mut _Movement>>,
		);
		app.insert_resource(input);

		app
	}

	mod run_to_walk {
		use super::*;

		#[test]
		fn call_update() {
			let mut app = setup(_Input::new().with_mock(|mock| {
				mock.expect_get_all_input_states::<MovementKey>()
					.returning(|| {
						Box::new(std::iter::once((
							MovementKey::ToggleWalkRun,
							InputState::just_pressed(),
						)))
					});
			}));
			app.world_mut().spawn((
				Player,
				PLAYER_RUN.clone(),
				_Movement::new().with_mock(|mock| {
					mock.expect_update()
						.once()
						.with(eq(PLAYER_WALK.speed), eq(PLAYER_WALK.animation.clone()))
						.return_const(());
				}),
			));

			app.update();
		}

		#[test]
		fn insert_walk() {
			let mut app = setup(_Input::new().with_mock(|mock| {
				mock.expect_get_all_input_states::<MovementKey>()
					.returning(|| {
						Box::new(std::iter::once((
							MovementKey::ToggleWalkRun,
							InputState::just_pressed(),
						)))
					});
			}));
			let entity = app
				.world_mut()
				.spawn((
					Player,
					PLAYER_RUN.clone(),
					_Movement::new().with_mock(|mock| {
						mock.expect_update().return_const(());
					}),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&*PLAYER_WALK),
				app.world().entity(entity).get::<MovementConfig>(),
			);
		}
	}

	mod walk_to_run {
		use super::*;

		#[test]
		fn call_update() {
			let mut app = setup(_Input::new().with_mock(|mock| {
				mock.expect_get_all_input_states::<MovementKey>()
					.returning(|| {
						Box::new(std::iter::once((
							MovementKey::ToggleWalkRun,
							InputState::just_pressed(),
						)))
					});
			}));
			app.world_mut().spawn((
				Player,
				PLAYER_WALK.clone(),
				_Movement::new().with_mock(|mock| {
					mock.expect_update()
						.once()
						.with(eq(PLAYER_RUN.speed), eq(PLAYER_RUN.animation.clone()))
						.return_const(());
				}),
			));

			app.update();
		}

		#[test]
		fn insert_run() {
			let mut app = setup(_Input::new().with_mock(|mock| {
				mock.expect_get_all_input_states::<MovementKey>()
					.returning(|| {
						Box::new(std::iter::once((
							MovementKey::ToggleWalkRun,
							InputState::just_pressed(),
						)))
					});
			}));
			let entity = app
				.world_mut()
				.spawn((
					Player,
					PLAYER_WALK.clone(),
					_Movement::new().with_mock(|mock| {
						mock.expect_update().return_const(());
					}),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&*PLAYER_RUN),
				app.world().entity(entity).get::<MovementConfig>(),
			);
		}
	}

	mod no_toggle_input {
		use super::*;

		#[test]
		fn do_not_call_update() {
			let mut app = setup(_Input::new().with_mock(|mock| {
				mock.expect_get_all_input_states::<MovementKey>()
					.returning(|| {
						Box::new(std::iter::once((
							MovementKey::ToggleWalkRun,
							InputState::pressed(),
						)))
					});
			}));
			app.world_mut().spawn((
				Player,
				PLAYER_WALK.clone(),
				_Movement::new().with_mock(|mock| {
					mock.expect_update().never();
				}),
			));

			app.update();
		}

		#[test]
		fn do_not_insert_config() {
			let mut app = setup(_Input::new().with_mock(|mock| {
				mock.expect_get_all_input_states::<MovementKey>()
					.returning(|| {
						Box::new(std::iter::once((
							MovementKey::ToggleWalkRun,
							InputState::pressed(),
						)))
					});
			}));
			let entity = app
				.world_mut()
				.spawn((
					Player,
					PLAYER_WALK.clone(),
					_Movement::new().with_mock(|mock| {
						mock.expect_update().return_const(());
					}),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&*PLAYER_WALK),
				app.world().entity(entity).get::<MovementConfig>(),
			);
		}
	}

	mod no_player {
		use super::*;

		#[test]
		fn do_not_call_update() {
			let mut app = setup(_Input::new().with_mock(|mock| {
				mock.expect_get_all_input_states::<MovementKey>()
					.returning(|| {
						Box::new(std::iter::once((
							MovementKey::ToggleWalkRun,
							InputState::just_pressed(),
						)))
					});
			}));
			app.world_mut().spawn((
				PLAYER_WALK.clone(),
				_Movement::new().with_mock(|mock| {
					mock.expect_update().never();
				}),
			));

			app.update();
		}

		#[test]
		fn do_not_insert_config() {
			let mut app = setup(_Input::new().with_mock(|mock| {
				mock.expect_get_all_input_states::<MovementKey>()
					.returning(|| {
						Box::new(std::iter::once((
							MovementKey::ToggleWalkRun,
							InputState::just_pressed(),
						)))
					});
			}));
			let entity = app
				.world_mut()
				.spawn((
					PLAYER_WALK.clone(),
					_Movement::new().with_mock(|mock| {
						mock.expect_update().return_const(());
					}),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&*PLAYER_WALK),
				app.world().entity(entity).get::<MovementConfig>(),
			);
		}
	}
}
