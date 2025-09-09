use crate::systems::movement::insert_process_component::{InputProcessComponent, ProcessInput};
use bevy::{ecs::query::QuerySingleError, prelude::*};
use common::{
	errors::{Error, Level},
	traits::{handles_player::KeyDirection, key_mappings::Pressed},
};
use std::{any::type_name, marker::PhantomData};

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
	fn parse<TCamera, TMap, TPlayer>(
		map: Res<TMap>,
		input: Res<ButtonInput<TMap::TInput>>,
		cameras: Query<&GlobalTransform, With<TCamera>>,
		players: Query<&Self::TInputProcessComponent, With<TPlayer>>,
	) -> Result<ProcessInput<Self>, TriggerMovementError<TCamera>>
	where
		TCamera: KeyDirection + Component,
		TMap: Pressed<TCamera::TKey> + Resource,
		TPlayer: Component,
	{
		let cam_transform = match cameras.single() {
			Err(QuerySingleError::NoEntities(_)) => {
				return Err(TriggerMovementError::from(QueryError::NoCam));
			}
			Err(QuerySingleError::MultipleEntities(_)) => {
				return Err(TriggerMovementError::from(QueryError::MultipleCams));
			}
			Ok(cam) => cam,
		};
		let direction: Vec3 = map
			.pressed(&input)
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

#[derive(Debug, PartialEq)]
pub(crate) struct TriggerMovementError<TCam> {
	_a: PhantomData<TCam>,
	value: QueryError,
}

impl<TCam> From<QueryError> for TriggerMovementError<TCam>
where
	TCam: Component,
{
	fn from(value: QueryError) -> Self {
		Self {
			_a: PhantomData,
			value,
		}
	}
}

impl<TCam> From<TriggerMovementError<TCam>> for Error
where
	TCam: Component,
{
	fn from(error: TriggerMovementError<TCam>) -> Self {
		match error.value {
			QueryError::NoCam => Error::Single {
				msg: format!("Found no camera of type {}", type_name::<TCam>()),
				lvl: Level::Error,
			},
			QueryError::MultipleCams => Error::Single {
				msg: format!("Found multiple cameras of type {}", type_name::<TCam>()),
				lvl: Level::Error,
			},
		}
	}
}

#[derive(Debug, PartialEq)]
pub(crate) enum QueryError {
	NoCam,
	MultipleCams,
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		tools::action_key::user_input::UserInput,
		traits::handles_player::DirectionError,
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::collections::HashSet;
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Component)]
	struct _Player;

	#[derive(Resource, NestedMocks)]
	struct _Map {
		mock: Mock_Map,
	}

	#[automock]
	impl Pressed<_Key> for _Map {
		type TInput = UserInput;

		fn pressed(&self, input: &ButtonInput<UserInput>) -> impl Iterator<Item = _Key> {
			self.mock.pressed(input)
		}
	}

	#[derive(Debug, PartialEq)]
	pub enum _Key {
		A,
		B,
		C,
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
	struct _Input(Dir3);

	impl InputProcessComponent for _Input {
		type TInputProcessComponent = _Component;
	}

	impl From<Dir3> for _Input {
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

	fn setup(map: _Map) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(map);
		app.init_resource::<ButtonInput<UserInput>>();

		app
	}

	#[test]
	fn trigger_single_direction_from_center() -> Result<(), RunSystemError> {
		mock_cam!();
		let cam_ctx = Mock_Cam::key_direction_context();
		cam_ctx.expect().returning(|_, _| Ok(Dir3::Z));
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_pressed()
				.returning(|_| Box::new(std::iter::once(_Key::A)));
		}));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));

		let input = app
			.world_mut()
			.run_system_once(_Input::parse::<_Cam, _Map, _Player>)?;

		assert_eq!(Ok(ProcessInput::New(_Input(Dir3::Z))), input);
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
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_pressed()
				.returning(|_| Box::new([_Key::A, _Key::B].into_iter()));
		}));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));

		let input = app
			.world_mut()
			.run_system_once(_Input::parse::<_Cam, _Map, _Player>)?;

		assert_eq!(
			Ok(ProcessInput::New(_Input(
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
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_pressed()
				.returning(|_| Box::new([_Key::A, _Key::B, _Key::C].into_iter()));
		}));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));

		let input = app
			.world_mut()
			.run_system_once(_Input::parse::<_Cam, _Map, _Player>)?;

		assert_eq!(Ok(ProcessInput::New(_Input(Dir3::X))), input);
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
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_pressed()
				.returning(|_| Box::new([].into_iter()));
		}));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));

		let input = app
			.world_mut()
			.run_system_once(_Input::parse::<_Cam, _Map, _Player>)?;

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
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_pressed()
				.returning(|_| Box::new([_Key::A, _Key::B].into_iter()));
		}));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));

		let input = app
			.world_mut()
			.run_system_once(_Input::parse::<_Cam, _Map, _Player>)?;

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
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_pressed()
				.returning(|_| Box::new([].into_iter()));
		}));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));
		app.world_mut().spawn((_Player, _Component::Directional));

		let input = app
			.world_mut()
			.run_system_once(_Input::parse::<_Cam, _Map, _Player>)?;

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
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_pressed()
				.returning(|_| Box::new([].into_iter()));
		}));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));
		app.world_mut().spawn(_Component::Directional);

		let input = app
			.world_mut()
			.run_system_once(_Input::parse::<_Cam, _Map, _Player>)?;

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
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_pressed()
				.returning(|_| Box::new([].into_iter()));
		}));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));
		app.world_mut().spawn((_Player, _Component::NotDirectional));

		let input = app
			.world_mut()
			.run_system_once(_Input::parse::<_Cam, _Map, _Player>)?;

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
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_pressed()
				.returning(move |_| Box::new(std::iter::once(_Key::A)));
		}));
		app.world_mut()
			.spawn((_Cam, GlobalTransform::from_xyz(1., 2., 3.)));

		_ = app
			.world_mut()
			.run_system_once(_Input::parse::<_Cam, _Map, _Player>)?;
		Ok(())
	}

	#[test]
	fn use_button_input() -> Result<(), RunSystemError> {
		mock_cam!();
		let cam_ctx = Mock_Cam::key_direction_context();
		cam_ctx.expect().returning(|_, _| Ok(Dir3::Z));
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_pressed().returning(move |input| {
				assert_eq!(
					HashSet::from([
						UserInput::from(KeyCode::Katakana),
						UserInput::from(KeyCode::Hiragana)
					]),
					input.get_just_pressed().copied().collect::<HashSet<_>>()
				);
				Box::new(std::iter::empty())
			});
		}));
		let mut input = ButtonInput::default();
		input.press(UserInput::from(KeyCode::Hiragana));
		input.press(UserInput::from(KeyCode::Katakana));
		app.insert_resource(input);
		app.world_mut().spawn((_Cam, GlobalTransform::default()));

		_ = app
			.world_mut()
			.run_system_once(_Input::parse::<_Cam, _Map, _Player>)?;
		Ok(())
	}

	#[test]
	fn return_no_cam_error() -> Result<(), RunSystemError> {
		mock_cam!();
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_pressed()
				.returning(|_| Box::new(std::iter::empty()));
		}));

		let result = app
			.world_mut()
			.run_system_once(_Input::parse::<_Cam, _Map, _Player>)?;

		assert_eq!(Err(TriggerMovementError::from(QueryError::NoCam)), result);
		Ok(())
	}

	#[test]
	fn return_multiple_cams_error() -> Result<(), RunSystemError> {
		mock_cam!();
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_pressed()
				.returning(|_| Box::new(std::iter::empty()));
		}));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));

		let result = app
			.world_mut()
			.run_system_once(_Input::parse::<_Cam, _Map, _Player>)?;

		assert_eq!(
			Err(TriggerMovementError::from(QueryError::MultipleCams)),
			result
		);
		Ok(())
	}
}
