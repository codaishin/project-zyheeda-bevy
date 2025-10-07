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
	pub(crate) fn update_actions<TInput, TRaycast>(
		input: StaticSystemParam<TInput>,
		raycast: StaticSystemParam<TRaycast>,
		mut players: Query<(&mut Actions, &Transform), With<Self>>,
		cameras: Query<&Transform, With<PlayerCamera>>,
	) where
		for<'w, 's> TInput: SystemParam<Item<'w, 's>: GetAllInputStates>,
		for<'w, 's> TRaycast: SystemParam<Item<'w, 's>: Raycast<Ground>>,
	{
		let cam_raycast = || {
			cameras
				.iter()
				.filter_map(|cam_transform| {
					let ray = Ray3d {
						origin: cam_transform.translation,
						direction: cam_transform.forward(),
					};
					let TimeOfImpact(toi) = raycast.raycast(ray, Ground)?;
					Some(ray.origin + ray.direction * toi)
				})
				.next()
		};

		let set_movement = |Actions(actions): &mut Actions, transform: &Transform| {
			for movement in input.get_all_input_states::<MovementKey>() {
				match movement {
					(MovementKey::Pointer, InputState::Pressed { .. }) => {
						if let Some(ground_target) = cam_raycast() {
							actions.insert(
								CurrentAction::Movement,
								AgentActionTarget::Point(ground_target),
							);
						}
					}
					(MovementKey::Forward, InputState::Pressed { .. }) => {
						actions.insert(
							CurrentAction::Movement,
							AgentActionTarget::Direction(transform.forward()),
						);
					}
					_ => {}
				}
			}
		};

		for (mut actions, transform) in &mut players {
			set_movement(&mut actions, transform);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{actions::Actions, player_camera::PlayerCamera};
	use common::{
		tools::action_key::{ActionKey, movement::MovementKey},
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
	impl Raycast<Ground> for _Raycast {
		fn raycast(&self, ray: Ray3d, constraints: Ground) -> Option<TimeOfImpact> {
			self.mock.raycast(ray, constraints)
		}
	}

	fn setup(input: _Input, raycast: _Raycast) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(input);
		app.insert_resource(raycast);
		app.add_systems(Update, Player::update_actions::<Res<_Input>, Res<_Raycast>>);

		app
	}

	#[test_case(InputState::pressed(), Transform::from_xyz(0., 1., 0.), TimeOfImpact(1.), Vec3::ZERO; "pressed")]
	#[test_case(InputState::just_pressed(), Transform::from_xyz(0., 1., 0.), TimeOfImpact(1.), Vec3::ZERO; "just pressed")]
	#[test_case(InputState::pressed(), Transform::from_xyz(1., 8., 3.), TimeOfImpact(5.), Vec3::new(1., 4., 6.); "offset")]
	fn move_to_point(input: InputState, cam_transform: Transform, toi: TimeOfImpact, target: Vec3) {
		let mut app = setup(
			_Input::new().with_mock(move |mock| {
				mock.expect_get_all_input_states()
					.returning(move || Box::new(std::iter::once((MovementKey::Pointer, input))));
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

	fn look_to(dir: Dir3) -> Transform {
		Transform::default().looking_to(dir, Dir3::Y)
	}

	#[test_case(InputState::pressed(), Dir3::NEG_Z, MovementKey::Forward; "forward")]
	#[test_case(InputState::just_pressed(), Dir3::NEG_Z, MovementKey::Forward; "just forward")]
	#[test_case(InputState::pressed(), Dir3::X, MovementKey::Forward; "forward offset")]
	fn move_to_direction(input: InputState, dir: Dir3, direction: MovementKey) {
		let mut app = setup(
			_Input::new().with_mock(move |mock| {
				mock.expect_get_all_input_states()
					.returning(move || Box::new(std::iter::once((direction, input))));
			}),
			_Raycast::new(),
		);
		let entity = app
			.world_mut()
			.spawn((Player, look_to(dir), Actions::default()))
			.id();
		app.world_mut().spawn((
			PlayerCamera,
			Transform::from_xyz(0., 1., 0.).looking_to(Vec3::ZERO, Dir3::Y),
		));

		app.update();

		assert_eq_approx!(
			Some(&Actions(HashMap::from([(
				CurrentAction::Movement,
				AgentActionTarget::Direction(dir),
			)]))),
			app.world().entity(entity).get::<Actions>(),
			0.0001,
		);
	}

	#[test]
	fn no_movement_when_cam_missing() {
		let mut app = setup(
			_Input::new().with_mock(move |mock| {
				mock.expect_get_all_input_states().returning(move || {
					Box::new(std::iter::once((
						MovementKey::Pointer,
						InputState::pressed(),
					)))
				});
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
}
