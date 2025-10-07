use crate::components::{actions::Actions, player::Player, player_camera::PlayerCamera};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	tools::action_key::movement::MovementKey,
	traits::{
		cast_ray::TimeOfImpact,
		handles_agents::{AgentActionTarget, CurrentAction},
		handles_input::{GetAllInputStates, InputState},
		handles_physics::{Ground, Raycast},
	},
};

impl Player {
	pub(crate) fn update_movement_actions<TInput, TRaycast>(
		input: StaticSystemParam<TInput>,
		raycast: StaticSystemParam<TRaycast>,
		mut players: Query<&mut Actions, With<Self>>,
		cameras: Query<&Transform, With<PlayerCamera>>,
	) where
		for<'w, 's> TInput: SystemParam<Item<'w, 's>: GetAllInputStates>,
		for<'w, 's> TRaycast: SystemParam<Item<'w, 's>: Raycast<Ground>>,
	{
		let Some(cam_transform) = cameras.iter().next() else {
			return;
		};
		let pressed_move_inputs = || {
			input
				.get_all_input_states::<MovementKey>()
				.filter_map(pressed)
		};
		let cam_raycast_to_ground = || {
			let ray = Ray3d {
				origin: cam_transform.translation,
				direction: cam_transform.forward(),
			};
			let TimeOfImpact(toi) = raycast.raycast(ray, Ground)?;
			Some(ray.origin + ray.direction * toi)
		};

		for mut actions in &mut players {
			let mut directions = Vec3::ZERO;
			let mut target = None;

			for key in pressed_move_inputs() {
				match key {
					MovementKey::Pointer => {
						target = cam_raycast_to_ground();
					}
					MovementKey::Forward => {
						directions += cam_transform
							.forward()
							.with_y(0.)
							.normalize_or(*cam_transform.up());
					}
					MovementKey::Backward => {
						directions += cam_transform
							.back()
							.with_y(0.)
							.normalize_or(*cam_transform.down());
					}
					MovementKey::Right => {
						directions += *cam_transform.right();
					}
					MovementKey::Left => {
						directions += *cam_transform.left();
					}
					_ => {}
				}
			}

			match (Dir3::try_from(directions), target) {
				(Ok(dir), _) => {
					actions.0.insert(MOVEMENT, AgentActionTarget::from(dir));
				}
				(_, Some(point)) => {
					actions.0.insert(MOVEMENT, AgentActionTarget::from(point));
				}
				_ if actions.0.contains_key(&MOVEMENT) => {
					actions.0.remove(&MOVEMENT);
				}
				_ => {}
			}
		}
	}
}

const MOVEMENT: CurrentAction = CurrentAction::Movement;

fn pressed((key, input_state): (MovementKey, InputState)) -> Option<MovementKey> {
	let InputState::Pressed { .. } = input_state else {
		return None;
	};

	Some(key)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{actions::Actions, player_camera::PlayerCamera};
	use MovementKey::{Backward, Forward, Left, Pointer, Right};
	use bevy::math::InvalidDirectionError;
	use common::{
		tools::action_key::{ActionKey, movement::MovementKey, slot::SlotKey},
		traits::{
			cast_ray::TimeOfImpact,
			handles_agents::{AgentActionTarget, CurrentAction},
			handles_input::InputState,
			iteration::IterFinite,
		},
	};
	use macros::NestedMocks;
	use mockall::automock;
	use std::collections::HashMap;
	use test_case::test_case;
	use testing::{IsChanged, NestedMocks, SingleThreadedApp, assert_eq_approx};

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
	impl Raycast<Ground> for _Raycast {
		fn raycast(&self, ray: Ray3d, constraints: Ground) -> Option<TimeOfImpact> {
			self.mock.raycast(ray, constraints)
		}
	}

	fn setup(input: _Input, raycast: _Raycast) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(input);
		app.insert_resource(raycast);
		app.add_systems(
			Update,
			(
				Player::update_movement_actions::<Res<_Input>, Res<_Raycast>>,
				IsChanged::<Actions>::detect,
			)
				.chain(),
		);

		app
	}

	#[test_case(InputState::pressed(), Transform::from_xyz(0., 1., 0.), TimeOfImpact(1.), Vec3::ZERO; "pressed")]
	#[test_case(InputState::just_pressed(), Transform::from_xyz(0., 1., 0.), TimeOfImpact(1.), Vec3::ZERO; "just pressed")]
	#[test_case(InputState::pressed(), Transform::from_xyz(1., 8., 3.), TimeOfImpact(5.), Vec3::new(1., 4., 6.); "offset")]
	fn move_to_point(input: InputState, cam_transform: Transform, toi: TimeOfImpact, target: Vec3) {
		let mut app = setup(
			_Input::new().with_mock(move |mock| {
				mock.expect_get_all_input_states()
					.returning(move || Box::new(std::iter::once((Pointer, input))));
			}),
			_Raycast::new().with_mock(|mock| {
				mock.expect_raycast().return_const(Some(toi));
			}),
		);
		let entity = app
			.world_mut()
			.spawn((Player, Transform::default(), Actions::default()))
			.id();
		app.world_mut()
			.spawn((PlayerCamera, cam_transform.looking_at(target, Dir3::Y)));

		app.update();

		assert_eq_approx!(
			Some(&Actions(HashMap::from([(
				CurrentAction::Movement,
				AgentActionTarget::Point(target)
			)]))),
			app.world().entity(entity).get::<Actions>(),
			0.0001,
		);
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
		let mut app = setup(
			_Input::new().with_mock(move |mock| {
				mock.expect_get_all_input_states()
					.returning(move || Box::new(std::iter::once((movement_key, input))));
			}),
			_Raycast::new(),
		);
		let entity = app
			.world_mut()
			.spawn((Player, Transform::default(), Actions::default()))
			.id();
		app.world_mut().spawn((
			PlayerCamera,
			Transform::from_xyz(0., 1., 0.).looking_to(cam_direction, Dir3::Y),
		));

		app.update();

		assert_eq_approx!(
			Some(&Actions(HashMap::from([(
				CurrentAction::Movement,
				AgentActionTarget::Direction(move_direction),
			)]))),
			app.world().entity(entity).get::<Actions>(),
			0.0001,
		);
	}

	#[test_case( Forward, Dir3::X, Dir3::X; "use cam up for forward")]
	#[test_case( Backward, Dir3::X, Dir3::NEG_X; "use cam down for backward")]
	fn when_directly_looking_down(movement_key: MovementKey, cam_up: Dir3, move_direction: Dir3) {
		let mut app = setup(
			_Input::new().with_mock(move |mock| {
				mock.expect_get_all_input_states().returning(move || {
					Box::new(std::iter::once((movement_key, InputState::pressed())))
				});
			}),
			_Raycast::new(),
		);
		let entity = app
			.world_mut()
			.spawn((Player, Transform::default(), Actions::default()))
			.id();
		app.world_mut().spawn((
			PlayerCamera,
			Transform::from_xyz(0., 1., 0.).looking_to(Dir3::NEG_Y, cam_up),
		));

		app.update();

		assert_eq_approx!(
			Some(&Actions(HashMap::from([(
				CurrentAction::Movement,
				AgentActionTarget::Direction(move_direction),
			)]))),
			app.world().entity(entity).get::<Actions>(),
			0.0001,
		);
	}

	#[test]
	fn sum_up_movement_directions() -> Result<(), InvalidDirectionError> {
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
		let entity = app
			.world_mut()
			.spawn((Player, Transform::default(), Actions::default()))
			.id();
		app.world_mut().spawn((
			PlayerCamera,
			Transform::from_xyz(0., 1., 0.).looking_to(Vec3::new(-1., -1., 0.), Dir3::Y),
		));

		app.update();

		assert_eq_approx!(
			Some(&Actions(HashMap::from([(
				CurrentAction::Movement,
				AgentActionTarget::Direction(Dir3::try_from(Vec3::new(-1., 0., -1.))?),
			)]))),
			app.world().entity(entity).get::<Actions>(),
			0.0001,
		);
		Ok(())
	}

	#[test_case([Forward, Backward]; "forward and back")]
	#[test_case([Left, Right]; "left and right")]
	fn ignore_move_direction_if_it_sums_up_to_zero(inputs: [MovementKey; 2]) {
		let mut app = setup(
			_Input::new().with_mock(move |mock| {
				mock.expect_get_all_input_states().returning(move || {
					Box::new(inputs.into_iter().map(|key| (key, InputState::pressed())))
				});
			}),
			_Raycast::new(),
		);
		let entity = app
			.world_mut()
			.spawn((Player, Transform::default(), Actions::default()))
			.id();
		app.world_mut().spawn((
			PlayerCamera,
			Transform::from_xyz(0., 1., 0.).looking_to(Vec3::new(-1., -1., 0.), Dir3::Y),
		));

		app.update();

		assert_eq!(
			Some(&Actions::default()),
			app.world().entity(entity).get::<Actions>(),
		);
	}

	#[test_case([Pointer, Forward]; "forward")]
	#[test_case([Pointer, Backward]; "back")]
	#[test_case([Pointer, Left]; "left")]
	#[test_case([Pointer, Right]; "right")]
	#[test_case([Forward, Pointer]; "first direction then pointer")]
	fn direction_movement_overrides_pointer_movement<const N: usize>(inputs: [MovementKey; N]) {
		let mut app = setup(
			_Input::new().with_mock(move |mock| {
				mock.expect_get_all_input_states().returning(move || {
					Box::new(inputs.into_iter().map(|key| (key, InputState::pressed())))
				});
			}),
			_Raycast::new().with_mock(|mock| {
				mock.expect_raycast().return_const(TimeOfImpact(42.));
			}),
		);
		let entity = app
			.world_mut()
			.spawn((Player, Transform::default(), Actions::default()))
			.id();
		app.world_mut().spawn((
			PlayerCamera,
			Transform::from_xyz(0., 1., 0.).looking_to(Vec3::new(-1., -1., 0.), Dir3::Y),
		));

		app.update();

		let move_action = app
			.world()
			.entity(entity)
			.get::<Actions>()
			.and_then(|a| a.0.get(&CurrentAction::Movement));
		assert!(
			matches!(move_action, Some(&AgentActionTarget::Direction(..)),),
			"{}expected direction, but was {move_action:?}{}",
			"\x1b[31m", // red on
			"\x1b[0m",  // red off
		);
	}

	#[test]
	fn set_movement_to_none_when_no_movement_input() {
		let mut app = setup(
			_Input::new().with_mock(move |mock| {
				mock.expect_get_all_input_states::<MovementKey>()
					.returning(move || Box::new(std::iter::empty()));
			}),
			_Raycast::new(),
		);
		let entity = app
			.world_mut()
			.spawn((
				Player,
				Transform::default(),
				Actions(HashMap::from([
					(CurrentAction::Movement, AgentActionTarget::Point(Vec3::ONE)),
					(
						CurrentAction::UseSkill(SlotKey(42)),
						AgentActionTarget::Point(Vec3::ONE),
					),
				])),
			))
			.id();
		app.world_mut().spawn((PlayerCamera, Transform::default()));

		app.update();

		assert_eq!(
			Some(&Actions(HashMap::from([(
				CurrentAction::UseSkill(SlotKey(42)),
				AgentActionTarget::Point(Vec3::ONE),
			)]))),
			app.world().entity(entity).get::<Actions>(),
		);
	}

	#[test]
	fn actions_component_unchanged_when_no_input() {
		let mut app = setup(
			_Input::new().with_mock(move |mock| {
				mock.expect_get_all_input_states::<MovementKey>()
					.returning(move || Box::new(std::iter::empty()));
			}),
			_Raycast::new(),
		);
		let entity = app
			.world_mut()
			.spawn((
				Player,
				Transform::default(),
				Actions(HashMap::from([(
					CurrentAction::UseSkill(SlotKey(42)),
					AgentActionTarget::Point(Vec3::ONE),
				)])),
			))
			.id();
		app.world_mut().spawn((PlayerCamera, Transform::default()));

		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world().entity(entity).get::<IsChanged<Actions>>(),
		);
	}

	#[test]
	fn no_movement_when_cam_missing() {
		let mut app = setup(
			_Input::new().with_mock(move |mock| {
				mock.expect_get_all_input_states()
					.returning(move || Box::new(std::iter::once((Pointer, InputState::pressed()))));
			}),
			_Raycast::new().with_mock(|mock| {
				mock.expect_raycast().return_const(Some(TimeOfImpact(42.)));
			}),
		);
		let entity = app
			.world_mut()
			.spawn((Player, Transform::default(), Actions::default()))
			.id();
		app.world_mut()
			.spawn(/* no camera component */ Transform::default());

		app.update();

		assert_eq_approx!(
			Some(&Actions::default()),
			app.world().entity(entity).get::<Actions>(),
			0.0001,
		);
	}

	#[test]
	fn no_movement_when_player_missing() {
		let mut app = setup(
			_Input::new().with_mock(move |mock| {
				mock.expect_get_all_input_states()
					.returning(move || Box::new(std::iter::once((Pointer, InputState::pressed()))));
			}),
			_Raycast::new().with_mock(|mock| {
				mock.expect_raycast().return_const(Some(TimeOfImpact(42.)));
			}),
		);
		let entity = app
			.world_mut()
			.spawn((
				/* no player component */ Transform::default(),
				Actions::default(),
			))
			.id();
		app.world_mut().spawn((PlayerCamera, Transform::default()));

		app.update();

		assert_eq_approx!(
			Some(&Actions::default()),
			app.world().entity(entity).get::<Actions>(),
			0.0001,
		);
	}
}
