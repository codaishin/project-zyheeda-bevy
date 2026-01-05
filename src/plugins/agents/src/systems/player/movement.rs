use crate::components::{
	movement_config::MovementConfig,
	player::Player,
	player_camera::PlayerCamera,
};
use MovementKey::{Backward, Forward, Left, Pointer, Right};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	tools::action_key::movement::MovementKey,
	traits::{
		accessors::get::GetContextMut,
		handles_input::{GetAllInputStates, InputState},
		handles_movement::{Movement, StartMovement, StopMovement},
		handles_physics::{MouseGroundHover, MouseGroundPoint, Raycast},
	},
};

impl Player {
	pub(crate) fn movement<TInput, TRaycast, TMovement>(
		mut m: StaticSystemParam<TMovement>,
		mut raycast: StaticSystemParam<TRaycast>,
		input: StaticSystemParam<TInput>,
		cameras: Query<&Transform, With<PlayerCamera>>,
		players: Query<(Entity, &MovementConfig), With<Self>>,
	) where
		for<'w, 's> TInput: SystemParam<Item<'w, 's>: GetAllInputStates>,
		for<'w, 's> TRaycast: SystemParam<Item<'w, 's>: Raycast<MouseGroundHover>>,
		for<'c> TMovement: GetContextMut<Movement, TContext<'c>: StartMovement + StopMovement>,
	{
		let Some(cam_transform) = cameras.iter().next() else {
			return;
		};
		let inputs = || input.get_all_input_states::<MovementKey>();

		for (entity, config) in &players {
			let Some(mut ctx) = TMovement::get_context_mut(&mut m, Movement { entity }) else {
				continue;
			};

			let mut directional_movement = DirectionalMovement::NotStopped;
			let mut directions = Vec3::ZERO;
			let mut target = None;

			for (key, state) in inputs() {
				match (key, state) {
					(Pointer, InputState::Pressed { .. }) => {
						target = raycast.raycast(MouseGroundHover);
					}
					(Forward, InputState::Pressed { .. }) => {
						directions += cam_transform
							.forward()
							.with_y(0.)
							.normalize_or(*cam_transform.up());
					}
					(Backward, InputState::Pressed { .. }) => {
						directions += cam_transform
							.back()
							.with_y(0.)
							.normalize_or(*cam_transform.down());
					}
					(Right, InputState::Pressed { .. }) => {
						directions += *cam_transform.right();
					}
					(Left, InputState::Pressed { .. }) => {
						directions += *cam_transform.left();
					}
					(
						Forward | Backward | Left | Right,
						InputState::Released { just_now: true },
					) => {
						directional_movement = DirectionalMovement::Stopped;
					}
					_ => {}
				}
			}

			match (Dir3::try_from(directions), target, directional_movement) {
				(Ok(dir), ..) => {
					ctx.start(dir, config.collider_radius, config.speed);
				}
				(_, Some(MouseGroundPoint(point)), DirectionalMovement::NotStopped) => {
					ctx.start(point, config.collider_radius, config.speed);
				}
				(Err(_), .., DirectionalMovement::Stopped) => {
					ctx.stop();
				}
				_ => {}
			}
		}
	}
}

enum DirectionalMovement {
	Stopped,
	NotStopped,
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::components::player_camera::PlayerCamera;
	use common::{
		tools::{
			Units,
			UnitsPerSecond,
			action_key::{ActionKey, movement::MovementKey},
		},
		traits::{
			handles_input::InputState,
			handles_movement::MovementTarget,
			iteration::IterFinite,
		},
	};
	use macros::NestedMocks;
	use mockall::{automock, mock, predicate::eq};
	use test_case::test_case;
	use testing::{NestedMocks, SingleThreadedApp, assert_eq_approx};

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

	#[derive(Resource, NestedMocks)]
	struct _Raycast {
		mock: Mock_Raycast,
	}

	#[automock]
	impl Raycast<MouseGroundHover> for _Raycast {
		fn raycast(&mut self, hover: MouseGroundHover) -> Option<MouseGroundPoint> {
			self.mock.raycast(hover)
		}
	}

	#[derive(Component, NestedMocks)]
	struct _Movement {
		mock: Mock_Movement,
	}

	impl StartMovement for _Movement {
		fn start<T>(&mut self, target: T, radius: Units, speed: UnitsPerSecond)
		where
			T: Into<MovementTarget> + 'static,
		{
			self.mock.start(target, radius, speed);
		}
	}

	impl StopMovement for _Movement {
		fn stop(&mut self) {
			self.mock.stop();
		}
	}

	mock! {
		_Movement {}
		impl StartMovement for _Movement {
			fn start<T>(
				&mut self,
				target: T,
				radius: Units,
				speed: UnitsPerSecond,
			) where T: Into<MovementTarget> + 'static;
		}
		impl StopMovement for _Movement {
			fn stop(&mut self);
		}
	}

	fn setup(input: _Input, raycast: _Raycast) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(input);
		app.insert_resource(raycast);
		app.add_systems(
			Update,
			Player::movement::<Res<_Input>, ResMut<_Raycast>, Query<&mut _Movement>>,
		);

		app
	}

	#[test_case(InputState::pressed(), Transform::from_xyz(0., 1., 0.), Vec3::ZERO; "pressed")]
	#[test_case(InputState::just_pressed(), Transform::from_xyz(0., 1., 0.), Vec3::ZERO; "just pressed")]
	#[test_case(InputState::pressed(), Transform::from_xyz(1., 8., 3.), Vec3::new(1., 4., 6.); "offset")]
	fn move_to_point(input: InputState, cam_transform: Transform, target: Vec3) {
		let collider_radius = Units::from(42.);
		let speed = UnitsPerSecond::from(11.);
		let mut app = setup(
			_Input::new().with_mock(move |mock| {
				mock.expect_get_all_input_states()
					.returning(move || Box::new(std::iter::once((Pointer, input))));
			}),
			_Raycast::new().with_mock(|mock| {
				mock.expect_raycast()
					.return_const(Some(MouseGroundPoint(target)));
			}),
		);
		app.world_mut().spawn((
			Player,
			Transform::default(),
			MovementConfig {
				collider_radius,
				speed,
			},
			_Movement::new().with_mock(move |mock| {
				mock.expect_start()
					.times(1)
					.with(eq(target), eq(collider_radius), eq(speed))
					.return_const(());
			}),
		));
		app.world_mut()
			.spawn((PlayerCamera, cam_transform.looking_at(target, Dir3::Y)));

		app.update();
	}

	#[test_case(InputState::pressed(), Forward, Vec3::new(-1., -1., 0.), Dir3::NEG_X; "forward pressed")]
	#[test_case(InputState::just_pressed(), Forward, Vec3::new(-1., -1., 0.), Dir3::NEG_X; "forward just pressed")]
	#[test_case(InputState::pressed(), Backward, Vec3::new(-1., -1., 0.), Dir3::X; "back pressed")]
	#[test_case(InputState::just_pressed(), Backward, Vec3::new(-1., -1., 0.), Dir3::X; "back just pressed")]
	#[test_case(InputState::pressed(), Right, Vec3::new(-1., -1., 0.), Dir3::NEG_Z; "right pressed")]
	#[test_case(InputState::just_pressed(), Right, Vec3::new(-1., -1., 0.), Dir3::NEG_Z; "right just pressed")]
	#[test_case(InputState::pressed(), Left, Vec3::new(-1., -1., 0.), Dir3::Z; "left pressed")]
	#[test_case(InputState::just_pressed(), Left, Vec3::new(-1., -1., 0.), Dir3::Z; "left just pressed")]
	fn move_to_direction(
		input: InputState,
		movement_key: MovementKey,
		cam_direction: Vec3,
		move_direction: Dir3,
	) {
		let collider_radius = Units::from(42.);
		let speed = UnitsPerSecond::from(11.);
		let mut app = setup(
			_Input::new().with_mock(move |mock| {
				mock.expect_get_all_input_states()
					.returning(move || Box::new(std::iter::once((movement_key, input))));
			}),
			_Raycast::new(),
		);
		app.world_mut().spawn((
			Player,
			Transform::default(),
			MovementConfig {
				collider_radius,
				speed,
			},
			_Movement::new().with_mock(move |mock| {
				mock.expect_start()
					.times(1)
					.with(eq(move_direction), eq(collider_radius), eq(speed))
					.return_const(());
			}),
		));
		app.world_mut().spawn((
			PlayerCamera,
			Transform::from_xyz(0., 1., 0.).looking_to(cam_direction, Dir3::Y),
		));

		app.update();
	}

	#[test_case( Forward, Dir3::X, Dir3::X; "use cam up for forward")]
	#[test_case( Backward, Dir3::X, Dir3::NEG_X; "use cam down for backward")]
	fn when_directly_looking_down(movement_key: MovementKey, cam_up: Dir3, move_direction: Dir3) {
		let collider_radius = Units::from(42.);
		let speed = UnitsPerSecond::from(11.);
		let mut app = setup(
			_Input::new().with_mock(move |mock| {
				mock.expect_get_all_input_states().returning(move || {
					Box::new(std::iter::once((movement_key, InputState::pressed())))
				});
			}),
			_Raycast::new(),
		);
		app.world_mut().spawn((
			Player,
			Transform::default(),
			MovementConfig {
				collider_radius,
				speed,
			},
			_Movement::new().with_mock(move |mock| {
				mock.expect_start()
					.times(1)
					.with(eq(move_direction), eq(collider_radius), eq(speed))
					.return_const(());
			}),
		));
		app.world_mut().spawn((
			PlayerCamera,
			Transform::from_xyz(0., 1., 0.).looking_to(Dir3::NEG_Y, cam_up),
		));

		app.update();
	}

	#[test]
	fn sum_up_movement_directions() {
		let expected = Dir3::try_from(Vec3::new(-1., 0., -1.)).unwrap();
		let collider_radius = Units::from(42.);
		let speed = UnitsPerSecond::from(11.);
		let mut app = setup(
			_Input::new().with_mock(move |mock| {
				mock.expect_get_all_input_states().returning(move || {
					Box::new(
						[Forward, Right]
							.into_iter()
							.map(|key| (key, InputState::pressed())),
					)
				});
			}),
			_Raycast::new(),
		);
		app.world_mut().spawn((
			Player,
			Transform::default(),
			MovementConfig {
				collider_radius,
				speed,
			},
			_Movement::new().with_mock(move |mock| {
				mock.expect_start::<Dir3>()
					.times(1)
					.withf(move |t, _, _| {
						assert_eq_approx!(expected, t, 0.001);
						true
					})
					.return_const(());
			}),
		));
		app.world_mut().spawn((
			PlayerCamera,
			Transform::from_xyz(0., 1., 0.).looking_to(Vec3::new(-1., -1., 0.), Dir3::Y),
		));

		app.update();
	}

	#[test_case([Forward, Backward]; "forward and back")]
	#[test_case([Left, Right]; "left and right")]
	fn ignore_move_direction_if_it_sums_up_to_zero(inputs: [MovementKey; 2]) {
		let collider_radius = Units::from(42.);
		let speed = UnitsPerSecond::from(11.);
		let mut app = setup(
			_Input::new().with_mock(move |mock| {
				mock.expect_get_all_input_states().returning(move || {
					Box::new(inputs.into_iter().map(|key| (key, InputState::pressed())))
				});
			}),
			_Raycast::new(),
		);
		app.world_mut().spawn((
			Player,
			Transform::default(),
			MovementConfig {
				collider_radius,
				speed,
			},
			_Movement::new().with_mock(move |mock| {
				mock.expect_start::<Dir3>().never();
				mock.expect_start::<Vec3>().never();
			}),
		));
		app.world_mut().spawn((
			PlayerCamera,
			Transform::from_xyz(0., 1., 0.).looking_to(Vec3::new(-1., -1., 0.), Dir3::Y),
		));

		app.update();
	}

	#[test_case([Pointer, Forward]; "forward")]
	#[test_case([Pointer, Backward]; "back")]
	#[test_case([Pointer, Left]; "left")]
	#[test_case([Pointer, Right]; "right")]
	#[test_case([Forward, Pointer]; "first direction then pointer")]
	fn direction_movement_overrides_pointer_movement<const N: usize>(inputs: [MovementKey; N]) {
		let collider_radius = Units::from(42.);
		let speed = UnitsPerSecond::from(11.);
		let mut app = setup(
			_Input::new().with_mock(move |mock| {
				mock.expect_get_all_input_states().returning(move || {
					Box::new(inputs.into_iter().map(|key| (key, InputState::pressed())))
				});
			}),
			_Raycast::new().with_mock(|mock| {
				mock.expect_raycast()
					.return_const(MouseGroundPoint(Vec3::new(4., 2., 1.)));
			}),
		);
		app.world_mut().spawn((
			Player,
			Transform::default(),
			MovementConfig {
				collider_radius,
				speed,
			},
			_Movement::new().with_mock(move |mock| {
				mock.expect_start::<Dir3>().times(1).return_const(());
				mock.expect_start::<Vec3>().never();
			}),
		));
		app.world_mut().spawn((
			PlayerCamera,
			Transform::from_xyz(0., 1., 0.).looking_to(Vec3::new(-1., -1., 0.), Dir3::Y),
		));

		app.update();
	}

	#[test_case(Forward; "forward")]
	#[test_case(Backward; "backward")]
	#[test_case(Left; "left")]
	#[test_case(Right; "right")]
	fn stop_movement_on_just_released_direction(input: MovementKey) {
		let collider_radius = Units::from(42.);
		let speed = UnitsPerSecond::from(11.);
		let mut app = setup(
			_Input::new().with_mock(move |mock| {
				mock.expect_get_all_input_states::<MovementKey>()
					.returning(move || {
						Box::new(
							[
								(input, InputState::just_released()),
								(Pointer, InputState::just_pressed()),
							]
							.into_iter(),
						)
					});
			}),
			_Raycast::new().with_mock(|mock| {
				mock.expect_raycast()
					.return_const(MouseGroundPoint(Vec3::ZERO));
			}),
		);
		app.world_mut().spawn((
			Player,
			Transform::default(),
			MovementConfig {
				collider_radius,
				speed,
			},
			_Movement::new().with_mock(move |mock| {
				mock.expect_start::<Vec3>().never();
				mock.expect_start::<Dir3>().never();
				mock.expect_stop().times(1).return_const(());
			}),
		));
		app.world_mut().spawn((PlayerCamera, Transform::default()));

		app.update();
	}

	#[test_case(Forward; "forward")]
	#[test_case(Backward; "backward")]
	#[test_case(Left; "left")]
	#[test_case(Right; "right")]
	fn no_stop_movement_on_released_direction(input: MovementKey) {
		let collider_radius = Units::from(42.);
		let speed = UnitsPerSecond::from(11.);
		let mut app = setup(
			_Input::new().with_mock(move |mock| {
				mock.expect_get_all_input_states::<MovementKey>()
					.returning(move || Box::new(std::iter::once((input, InputState::released()))));
			}),
			_Raycast::new(),
		);
		app.world_mut().spawn((
			Player,
			Transform::default(),
			MovementConfig {
				collider_radius,
				speed,
			},
			_Movement::new().with_mock(move |mock| {
				mock.expect_stop().never();
			}),
		));
		app.world_mut().spawn((PlayerCamera, Transform::default()));

		app.update();
	}

	#[test_case(Forward, Left; "forward")]
	#[test_case(Backward, Right; "backward")]
	#[test_case(Left, Forward; "left")]
	#[test_case(Right, Backward; "right")]
	fn no_stop_movement_on_released_direction_when_other_direction_pressed(
		input: MovementKey,
		other_input: MovementKey,
	) {
		let collider_radius = Units::from(42.);
		let speed = UnitsPerSecond::from(11.);
		let mut app = setup(
			_Input::new().with_mock(move |mock| {
				mock.expect_get_all_input_states::<MovementKey>()
					.returning(move || {
						Box::new(
							[
								(input, InputState::just_released()),
								(other_input, InputState::pressed()),
							]
							.into_iter(),
						)
					});
			}),
			_Raycast::new(),
		);
		app.world_mut().spawn((
			Player,
			Transform::default(),
			MovementConfig {
				collider_radius,
				speed,
			},
			_Movement::new().with_mock(move |mock| {
				mock.expect_stop().never();
				mock.expect_start::<Vec3>().return_const(());
				mock.expect_start::<Dir3>().return_const(());
			}),
		));
		app.world_mut().spawn((PlayerCamera, Transform::default()));

		app.update();
	}

	#[test]
	fn no_movement_when_cam_missing() {
		let collider_radius = Units::from(42.);
		let speed = UnitsPerSecond::from(11.);
		let mut app = setup(
			_Input::new().with_mock(move |mock| {
				mock.expect_get_all_input_states()
					.returning(move || Box::new(std::iter::once((Pointer, InputState::pressed()))));
			}),
			_Raycast::new().with_mock(|mock| {
				mock.expect_raycast()
					.return_const(MouseGroundPoint(Vec3::ZERO));
			}),
		);
		app.world_mut().spawn((
			Player,
			Transform::default(),
			MovementConfig {
				collider_radius,
				speed,
			},
			_Movement::new().with_mock(move |mock| {
				mock.expect_start::<Vec3>().never();
				mock.expect_start::<Dir3>().never();
			}),
		));
		app.world_mut()
			.spawn(/* no camera component */ Transform::default());

		app.update();
	}

	#[test]
	fn no_movement_when_player_missing() {
		let collider_radius = Units::from(42.);
		let speed = UnitsPerSecond::from(11.);
		let mut app = setup(
			_Input::new().with_mock(move |mock| {
				mock.expect_get_all_input_states()
					.returning(move || Box::new(std::iter::once((Pointer, InputState::pressed()))));
			}),
			_Raycast::new().with_mock(|mock| {
				mock.expect_raycast()
					.return_const(MouseGroundPoint(Vec3::ZERO));
			}),
		);
		app.world_mut().spawn((
			/* No player */
			Transform::default(),
			MovementConfig {
				collider_radius,
				speed,
			},
			_Movement::new().with_mock(move |mock| {
				mock.expect_start::<Vec3>().never();
				mock.expect_start::<Dir3>().never();
			}),
		));
		app.world_mut().spawn((PlayerCamera, Transform::default()));

		app.update();
	}
}
