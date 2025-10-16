use crate::systems::movement::insert_process_component::{InputProcessComponent, ProcessInput};
use bevy::{
	ecs::{
		query::QuerySingleError,
		system::{StaticSystemParam, SystemParam},
	},
	prelude::*,
};
use common::{
	errors::UniqueViolation,
	tools::action_key::ActionKey,
	traits::{
		handles_input::{GetAllInputStates, InputState},
		handles_player::KeyDirection,
		iteration::IterFinite,
	},
};

impl<T> ParseDirectionalMovement for T
where
	T: From<Dir3> + InputProcessComponent,
	Self::TInputProcessComponent: UsesDirection,
{
}

pub(crate) trait ParseDirectionalMovement: From<Dir3> + InputProcessComponent
where
	Self::TInputProcessComponent: UsesDirection,
{
	fn parse<TCamera, TInput, TPlayer>(
		input: StaticSystemParam<TInput>,
		cameras: Query<&GlobalTransform, With<TCamera>>,
		players: Query<&Self::TInputProcessComponent, With<TPlayer>>,
	) -> Result<ProcessInput<Self>, UniqueViolation<TCamera>>
	where
		TCamera: KeyDirection + Component,
		TCamera::TKey: IterFinite + Into<ActionKey> + 'static,
		for<'w, 's> TInput: SystemParam<Item<'w, 's>: GetAllInputStates>,
		TPlayer: Component,
	{
		let cam_transform = match cameras.single() {
			Err(QuerySingleError::NoEntities(_)) => {
				return Err(UniqueViolation::none());
			}
			Err(QuerySingleError::MultipleEntities(_)) => {
				return Err(UniqueViolation::multiple());
			}
			Ok(cam) => cam,
		};
		let direction: Vec3 = input
			.get_all_input_states::<TCamera::TKey>()
			.filter_map(|(movement, state)| match state {
				InputState::Pressed { .. } => Some(movement),
				_ => None,
			})
			.filter_map(|key| TCamera::key_direction(cam_transform, &key).ok())
			.map(|d| *d)
			.sum();

		match Dir3::try_from(direction) {
			Ok(direction) => Ok(ProcessInput::New(Self::from(direction))),
			Err(_) => Ok(stop_or_none(players)),
		}
	}
}

fn stop_or_none<T, TComponent, TPlayer>(
	players: Query<&TComponent, With<TPlayer>>,
) -> ProcessInput<T>
where
	TComponent: Component + UsesDirection,
	TPlayer: Component,
{
	match players.single() {
		Ok(player) if player.uses_direction() => ProcessInput::Stop,
		_ => ProcessInput::None,
	}
}

pub(crate) trait UsesDirection {
	fn uses_direction(&self) -> bool;
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::traits::{handles_player::DirectionError, iteration::Iter};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Component)]
	struct _Player;

	#[derive(SystemParam)]
	struct _InputParam<'w> {
		input: Res<'w, _Input>,
	}

	impl GetAllInputStates for _InputParam<'_> {
		fn get_all_input_states<TAction>(&self) -> impl Iterator<Item = (TAction, InputState)>
		where
			TAction: Into<ActionKey> + IterFinite + 'static,
		{
			self.input.get_all_input_states()
		}
	}

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

	#[derive(Debug, PartialEq, Clone, Copy)]
	pub enum _Key {
		A,
		B,
		C,
	}

	impl From<_Key> for ActionKey {
		fn from(_: _Key) -> Self {
			panic!("NOT USED")
		}
	}

	impl IterFinite for _Key {
		fn iterator() -> Iter<Self> {
			panic!("NOT USED")
		}

		fn next(_: &Iter<Self>) -> Option<Self> {
			panic!("NOT USED")
		}
	}

	#[derive(Component)]
	struct _Cam;

	#[automock]
	impl KeyDirection for _Cam {
		type TKey = _Key;

		fn key_direction(
			self_transform: &GlobalTransform,
			movement_key: &_Key,
		) -> Result<Dir3, DirectionError<_Key>> {
			Mock_Cam::key_direction(self_transform, movement_key)
		}
	}

	#[derive(Component)]
	enum _Component {
		Directional,
		NotDirectional,
	}

	impl UsesDirection for _Component {
		fn uses_direction(&self) -> bool {
			matches!(self, _Component::Directional)
		}
	}

	#[derive(Debug, PartialEq, Clone, Copy)]
	struct _Result(Dir3);

	impl InputProcessComponent for _Result {
		type TInputProcessComponent = _Component;
	}

	impl From<Dir3> for _Result {
		fn from(direction: Dir3) -> Self {
			Self(direction)
		}
	}

	macro_rules! mock_cam {
		() => {
			#[derive(Component, Debug, PartialEq)]
			struct _Cam;

			#[automock]
			impl KeyDirection for _Cam {
				type TKey = _Key;

				fn key_direction(
					self_transform: &GlobalTransform,
					movement_key: &_Key,
				) -> Result<Dir3, DirectionError<_Key>> {
					Mock_Cam::key_direction(self_transform, movement_key)
				}
			}
		};
	}

	fn setup(map: _Input) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(map);

		app
	}

	#[test]
	fn trigger_single_direction_from_center_pressed() -> Result<(), RunSystemError> {
		mock_cam!();
		let cam_ctx = Mock_Cam::key_direction_context();
		cam_ctx.expect().returning(|_, _| Ok(Dir3::Z));
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states()
				.returning(|| Box::new(std::iter::once((_Key::A, InputState::pressed()))));
		}));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));

		let input = app
			.world_mut()
			.run_system_once(_Result::parse::<_Cam, _InputParam, _Player>)?;

		assert_eq!(Ok(ProcessInput::New(_Result(Dir3::Z))), input);
		Ok(())
	}

	#[test]
	fn trigger_single_direction_from_center_just_pressed() -> Result<(), RunSystemError> {
		mock_cam!();
		let cam_ctx = Mock_Cam::key_direction_context();
		cam_ctx.expect().returning(|_, _| Ok(Dir3::Z));
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states()
				.returning(|| Box::new(std::iter::once((_Key::A, InputState::just_pressed()))));
		}));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));

		let input = app
			.world_mut()
			.run_system_once(_Result::parse::<_Cam, _InputParam, _Player>)?;

		assert_eq!(Ok(ProcessInput::New(_Result(Dir3::Z))), input);
		Ok(())
	}

	#[test]
	fn trigger_nothing_when_released() -> Result<(), RunSystemError> {
		mock_cam!();
		let cam_ctx = Mock_Cam::key_direction_context();
		cam_ctx.expect().returning(|_, _| Ok(Dir3::Z));
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states().returning(|| {
				Box::new(
					[
						(_Key::A, InputState::released()),
						(_Key::B, InputState::just_released()),
					]
					.into_iter(),
				)
			});
		}));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));

		let input = app
			.world_mut()
			.run_system_once(_Result::parse::<_Cam, _InputParam, _Player>)?;

		assert_eq!(Ok(ProcessInput::None), input);
		Ok(())
	}

	#[test]
	fn trigger_accumulated_2_direction_from_center() -> Result<(), RunSystemError> {
		mock_cam!();
		let cam_ctx = Mock_Cam::key_direction_context();
		cam_ctx
			.expect()
			.with(eq(GlobalTransform::default()), eq(_Key::A))
			.returning(|_, _| Ok(Dir3::Z));
		cam_ctx
			.expect()
			.with(eq(GlobalTransform::default()), eq(_Key::B))
			.returning(|_, _| Ok(Dir3::X));
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states().returning(|| {
				Box::new(
					[
						(_Key::A, InputState::pressed()),
						(_Key::B, InputState::pressed()),
					]
					.into_iter(),
				)
			});
		}));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));

		let input = app
			.world_mut()
			.run_system_once(_Result::parse::<_Cam, _InputParam, _Player>)?;

		assert_eq!(
			Ok(ProcessInput::New(_Result(
				Dir3::try_from(Vec3::Z + Vec3::X).unwrap()
			))),
			input
		);
		Ok(())
	}

	#[test]
	fn trigger_accumulated_3_direction_from_center() -> Result<(), RunSystemError> {
		mock_cam!();
		let cam_ctx = Mock_Cam::key_direction_context();
		cam_ctx
			.expect()
			.with(eq(GlobalTransform::default()), eq(_Key::A))
			.returning(|_, _| Ok(Dir3::Z));
		cam_ctx
			.expect()
			.with(eq(GlobalTransform::default()), eq(_Key::B))
			.returning(|_, _| Ok(Dir3::X));
		cam_ctx
			.expect()
			.with(eq(GlobalTransform::default()), eq(_Key::C))
			.returning(|_, _| Ok(Dir3::NEG_Z));
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states().returning(|| {
				Box::new(
					[
						(_Key::A, InputState::pressed()),
						(_Key::B, InputState::pressed()),
						(_Key::C, InputState::pressed()),
					]
					.into_iter(),
				)
			});
		}));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));

		let input = app
			.world_mut()
			.run_system_once(_Result::parse::<_Cam, _InputParam, _Player>)?;

		assert_eq!(Ok(ProcessInput::New(_Result(Dir3::X))), input);
		Ok(())
	}

	#[test]
	fn none_when_no_directions() -> Result<(), RunSystemError> {
		mock_cam!();
		let cam_ctx = Mock_Cam::key_direction_context();
		cam_ctx
			.expect()
			.with(eq(GlobalTransform::default()), eq(_Key::A))
			.returning(|_, _| Ok(Dir3::Z));
		cam_ctx
			.expect()
			.with(eq(GlobalTransform::default()), eq(_Key::B))
			.returning(|_, _| Ok(Dir3::NEG_Z));
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states::<_Key>()
				.returning(|| Box::new(std::iter::empty()));
		}));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));

		let input = app
			.world_mut()
			.run_system_once(_Result::parse::<_Cam, _InputParam, _Player>)?;

		assert_eq!(Ok(ProcessInput::None), input);
		Ok(())
	}

	#[test]
	fn none_when_accumulated_2_directions_are_zero_from_center() -> Result<(), RunSystemError> {
		mock_cam!();
		let cam_ctx = Mock_Cam::key_direction_context();
		cam_ctx
			.expect()
			.with(eq(GlobalTransform::default()), eq(_Key::A))
			.returning(|_, _| Ok(Dir3::Z));
		cam_ctx
			.expect()
			.with(eq(GlobalTransform::default()), eq(_Key::B))
			.returning(|_, _| Ok(Dir3::NEG_Z));
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states().returning(|| {
				Box::new(
					[
						(_Key::A, InputState::pressed()),
						(_Key::B, InputState::pressed()),
					]
					.into_iter(),
				)
			});
		}));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));

		let input = app
			.world_mut()
			.run_system_once(_Result::parse::<_Cam, _InputParam, _Player>)?;

		assert_eq!(Ok(ProcessInput::None), input);
		Ok(())
	}

	#[test]
	fn stop_when_no_directions_and_player_process_component_is_directional()
	-> Result<(), RunSystemError> {
		mock_cam!();
		let cam_ctx = Mock_Cam::key_direction_context();
		cam_ctx
			.expect()
			.with(eq(GlobalTransform::default()), eq(_Key::A))
			.returning(|_, _| Ok(Dir3::Z));
		cam_ctx
			.expect()
			.with(eq(GlobalTransform::default()), eq(_Key::B))
			.returning(|_, _| Ok(Dir3::NEG_Z));
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states::<_Key>()
				.returning(|| Box::new(std::iter::empty()));
		}));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));
		app.world_mut().spawn((_Player, _Component::Directional));

		let input = app
			.world_mut()
			.run_system_once(_Result::parse::<_Cam, _InputParam, _Player>)?;

		assert_eq!(Ok(ProcessInput::Stop), input);
		Ok(())
	}

	#[test]
	fn none_when_no_directions_and_non_player_process_component_is_directional()
	-> Result<(), RunSystemError> {
		mock_cam!();
		let cam_ctx = Mock_Cam::key_direction_context();
		cam_ctx
			.expect()
			.with(eq(GlobalTransform::default()), eq(_Key::A))
			.returning(|_, _| Ok(Dir3::Z));
		cam_ctx
			.expect()
			.with(eq(GlobalTransform::default()), eq(_Key::B))
			.returning(|_, _| Ok(Dir3::NEG_Z));
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states::<_Key>()
				.returning(|| Box::new(std::iter::empty()));
		}));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));
		app.world_mut().spawn(_Component::Directional);

		let input = app
			.world_mut()
			.run_system_once(_Result::parse::<_Cam, _InputParam, _Player>)?;

		assert_eq!(Ok(ProcessInput::None), input);
		Ok(())
	}

	#[test]
	fn none_when_no_directions_and_player_process_component_is_not_directional()
	-> Result<(), RunSystemError> {
		mock_cam!();
		let cam_ctx = Mock_Cam::key_direction_context();
		cam_ctx
			.expect()
			.with(eq(GlobalTransform::default()), eq(_Key::A))
			.returning(|_, _| Ok(Dir3::Z));
		cam_ctx
			.expect()
			.with(eq(GlobalTransform::default()), eq(_Key::B))
			.returning(|_, _| Ok(Dir3::NEG_Z));
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states::<_Key>()
				.returning(|| Box::new(std::iter::empty()));
		}));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));
		app.world_mut().spawn((_Player, _Component::NotDirectional));

		let input = app
			.world_mut()
			.run_system_once(_Result::parse::<_Cam, _InputParam, _Player>)?;

		assert_eq!(Ok(ProcessInput::None), input);
		Ok(())
	}

	#[test]
	fn use_camera_transform() -> Result<(), RunSystemError> {
		mock_cam!();
		let cam_ctx = Mock_Cam::key_direction_context();
		cam_ctx
			.expect()
			.with(eq(GlobalTransform::from_xyz(1., 2., 3.)), eq(_Key::A))
			.returning(|_, _| Ok(Dir3::Z));
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states::<_Key>()
				.returning(|| Box::new(std::iter::once((_Key::A, InputState::pressed()))));
		}));
		app.world_mut()
			.spawn((_Cam, GlobalTransform::from_xyz(1., 2., 3.)));

		_ = app
			.world_mut()
			.run_system_once(_Result::parse::<_Cam, _InputParam, _Player>)?;
		Ok(())
	}

	#[test]
	fn return_no_cam_error() -> Result<(), RunSystemError> {
		mock_cam!();
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states::<_Key>()
				.returning(|| Box::new(std::iter::empty()));
		}));

		let result = app
			.world_mut()
			.run_system_once(_Result::parse::<_Cam, _InputParam, _Player>)?;

		assert_eq!(Err(UniqueViolation::none()), result);
		Ok(())
	}

	#[test]
	fn return_multiple_cams_error() -> Result<(), RunSystemError> {
		mock_cam!();
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states::<_Key>()
				.returning(|| Box::new(std::iter::empty()));
		}));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));

		let result = app
			.world_mut()
			.run_system_once(_Result::parse::<_Cam, _InputParam, _Player>)?;

		assert_eq!(Err(UniqueViolation::multiple()), result);
		Ok(())
	}
}
