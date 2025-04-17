use crate::traits::change_per_frame::MinDistance;
use bevy::{ecs::query::QuerySingleError, prelude::*};
use common::{
	errors::{Error, Level},
	tools::speed::Speed,
	traits::{accessors::get::Getter, handles_player::KeyDirection, key_mappings::Pressed},
};
use std::{any::type_name, marker::PhantomData, time::Duration};

impl<T> TriggerDirectionKeyMovement for T where T: From<Vec3> + Event + MinDistance {}

pub(crate) trait TriggerDirectionKeyMovement: From<Vec3> + Event + MinDistance {
	fn trigger_movement<TCamera, TAgent, TMap, TKey>(
		In(delta): In<Duration>,
		map: Res<TMap>,
		input: Res<ButtonInput<KeyCode>>,
		mut events: EventWriter<Self>,
		agents: Query<(&GlobalTransform, &TAgent)>,
		cameras: Query<&GlobalTransform, With<TCamera>>,
	) -> Result<(), TriggerMovementError<TCamera, TAgent>>
	where
		TCamera: KeyDirection<TKey> + Component,
		TAgent: Getter<Speed> + Component,
		TMap: Pressed<TKey, KeyCode> + Resource,
	{
		let cam_transform = match cameras.get_single() {
			Err(QuerySingleError::NoEntities(_)) => {
				return Err(TriggerMovementError::from(QueryError::NoCam));
			}
			Err(QuerySingleError::MultipleEntities(_)) => {
				return Err(TriggerMovementError::from(QueryError::MultipleCams));
			}
			Ok(cam) => cam,
		};
		let (transform, speed) = match agents.get_single() {
			Err(QuerySingleError::NoEntities(_)) => {
				return Err(TriggerMovementError::from(QueryError::NoAgent));
			}
			Err(QuerySingleError::MultipleEntities(_)) => {
				return Err(TriggerMovementError::from(QueryError::MultipleAgents));
			}
			Ok(agent) => agent,
		};

		let translation = transform.translation();
		let direction = map
			.pressed(&input)
			.filter_map(|key| TCamera::key_direction(cam_transform, &key).ok())
			.fold(Vec3::ZERO, |a, b| a + *b);

		let Some(direction) = direction.try_normalize() else {
			return Ok(());
		};

		events.send(Self::from(
			translation + direction * Self::min_distance(speed.get(), delta) * SCALE_MIN_DISTANCE,
		));

		Ok(())
	}
}

/// Used to compensate fluctuations in update deltas by overestimating the minimal distance
/// that can be traveled per frame with the current speed.
const SCALE_MIN_DISTANCE: f32 = 1.5;

#[derive(Debug, PartialEq)]
pub(crate) struct TriggerMovementError<TCam, TAgent>
where
	TAgent: Component,
{
	_a: PhantomData<(TCam, TAgent)>,
	value: QueryError,
}

impl<TCam, TAgent> From<QueryError> for TriggerMovementError<TCam, TAgent>
where
	TCam: Component,
	TAgent: Component,
{
	fn from(value: QueryError) -> Self {
		Self {
			_a: PhantomData,
			value,
		}
	}
}

impl<TCam, TAgent> From<TriggerMovementError<TCam, TAgent>> for Error
where
	TCam: Component,
	TAgent: Component,
{
	fn from(error: TriggerMovementError<TCam, TAgent>) -> Self {
		match error.value {
			QueryError::NoAgent => Error {
				msg: format!("Found no agent of type {}", type_name::<TAgent>()),
				lvl: Level::Error,
			},
			QueryError::MultipleAgents => Error {
				msg: format!("Found multiple agents of type {}", type_name::<TAgent>()),
				lvl: Level::Error,
			},
			QueryError::NoCam => Error {
				msg: format!("Found no camera of type {}", type_name::<TCam>()),
				lvl: Level::Error,
			},
			QueryError::MultipleCams => Error {
				msg: format!("Found multiple cameras of type {}", type_name::<TCam>()),
				lvl: Level::Error,
			},
		}
	}
}

#[derive(Debug, PartialEq)]
pub(crate) enum QueryError {
	NoAgent,
	MultipleAgents,
	NoCam,
	MultipleCams,
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		math::InvalidDirectionError,
	};
	use common::{
		test_tools::utils::SingleThreadedApp,
		tools::UnitsPerSecond,
		traits::{clamp_zero_positive::ClampZeroPositive, nested_mock::NestedMocks},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::collections::HashSet;

	#[derive(Resource, NestedMocks)]
	struct _Map {
		mock: Mock_Map,
	}

	#[automock]
	impl Pressed<_Key, KeyCode> for _Map {
		fn pressed(&self, input: &ButtonInput<KeyCode>) -> impl Iterator<Item = _Key> {
			self.mock.pressed(input)
		}
	}

	#[derive(Debug, PartialEq)]
	enum _Key {
		A,
		B,
		C,
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Agent(Speed);

	impl Getter<Speed> for _Agent {
		fn get(&self) -> Speed {
			self.0
		}
	}

	#[derive(Component)]
	struct _Cam;

	#[automock]
	impl KeyDirection<_Key> for _Cam {
		fn key_direction(
			self_transform: &GlobalTransform,
			movement_key: &_Key,
		) -> Result<Dir3, InvalidDirectionError> {
			Mock_Cam::key_direction(self_transform, movement_key)
		}
	}

	macro_rules! mock_event {
		() => {
			#[derive(Event, Debug, PartialEq, Clone, Copy)]
			struct _Event(Vec3);

			#[automock]
			impl MinDistance for _Event {
				fn min_distance(speed: Speed, delta: Duration) -> f32 {
					Mock_Event::min_distance(speed, delta)
				}
			}

			impl From<Vec3> for _Event {
				fn from(translation: Vec3) -> Self {
					Self(translation)
				}
			}
		};
	}

	macro_rules! mock_cam {
		() => {
			#[derive(Component, Debug, PartialEq)]
			struct _Cam;

			#[automock]
			impl KeyDirection<_Key> for _Cam {
				fn key_direction(
					self_transform: &GlobalTransform,
					movement_key: &_Key,
				) -> Result<Dir3, InvalidDirectionError> {
					Mock_Cam::key_direction(self_transform, movement_key)
				}
			}
		};
	}

	fn move_input_events<TEvent>(app: &App) -> Vec<TEvent>
	where
		TEvent: Event + Copy,
	{
		let events = app.world().resource::<Events<TEvent>>();
		let mut cursor = events.get_cursor();

		cursor.read(events).copied().collect()
	}

	fn setup<TEvent>(map: _Map) -> App
	where
		TEvent: Event,
	{
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(map);
		app.init_resource::<ButtonInput<KeyCode>>();
		app.add_event::<TEvent>();

		app
	}

	#[test]
	fn trigger_single_direction_from_center() -> Result<(), RunSystemError> {
		mock_event!();
		let event_ctx = Mock_Event::min_distance_context();
		event_ctx.expect().return_const(1. / SCALE_MIN_DISTANCE);
		mock_cam!();
		let cam_ctx = Mock_Cam::key_direction_context();
		cam_ctx.expect().returning(|_, _| Ok(Dir3::Z));
		let mut app = setup::<_Event>(_Map::new().with_mock(|mock| {
			mock.expect_pressed()
				.returning(|_| Box::new(std::iter::once(_Key::A)));
		}));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));
		app.world_mut().spawn((
			GlobalTransform::default(),
			_Agent(Speed(UnitsPerSecond::new(1.))),
		));

		_ = app.world_mut().run_system_once_with(
			Duration::from_secs(1),
			_Event::trigger_movement::<_Cam, _Agent, _Map, _Key>,
		)?;

		assert_eq!(vec![_Event(Vec3::Z)], move_input_events(&app));
		Ok(())
	}

	#[test]
	fn call_min_distance_with_delta_and_agent_speed() -> Result<(), RunSystemError> {
		let speed = Speed(UnitsPerSecond::new(42.));
		let delta = Duration::from_secs(5);
		mock_event!();
		let event_ctx = Mock_Event::min_distance_context();
		event_ctx
			.expect()
			.with(eq(speed), eq(delta))
			.return_const(1. / SCALE_MIN_DISTANCE);
		mock_cam!();
		let cam_ctx = Mock_Cam::key_direction_context();
		cam_ctx.expect().returning(|_, _| Ok(Dir3::Z));
		let mut app = setup::<_Event>(_Map::new().with_mock(|mock| {
			mock.expect_pressed()
				.returning(|_| Box::new(std::iter::once(_Key::A)));
		}));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));
		app.world_mut()
			.spawn((GlobalTransform::default(), _Agent(speed)));

		_ = app
			.world_mut()
			.run_system_once_with(delta, _Event::trigger_movement::<_Cam, _Agent, _Map, _Key>)?;
		Ok(())
	}

	#[test]
	fn scale_direction_with_min_distance_and_scale() -> Result<(), RunSystemError> {
		mock_event!();
		let min_distance = 4.;
		let event_ctx = Mock_Event::min_distance_context();
		event_ctx.expect().return_const(min_distance);
		mock_cam!();
		let cam_ctx = Mock_Cam::key_direction_context();
		cam_ctx.expect().returning(|_, _| Ok(Dir3::Z));
		let mut app = setup::<_Event>(_Map::new().with_mock(|mock| {
			mock.expect_pressed()
				.returning(|_| Box::new(std::iter::once(_Key::A)));
		}));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));
		app.world_mut().spawn((
			GlobalTransform::default(),
			_Agent(Speed(UnitsPerSecond::new(42.))),
		));

		_ = app.world_mut().run_system_once_with(
			Duration::from_secs(1),
			_Event::trigger_movement::<_Cam, _Agent, _Map, _Key>,
		)?;

		assert_eq!(
			vec![_Event(Vec3::Z * min_distance * SCALE_MIN_DISTANCE)],
			move_input_events(&app)
		);
		Ok(())
	}

	#[test]
	fn trigger_single_direction_from_offset() -> Result<(), RunSystemError> {
		mock_event!();
		let event_ctx = Mock_Event::min_distance_context();
		event_ctx.expect().return_const(1. / SCALE_MIN_DISTANCE);
		mock_cam!();
		let cam_ctx = Mock_Cam::key_direction_context();
		cam_ctx.expect().returning(|_, _| Ok(Dir3::Z));
		let mut app = setup::<_Event>(_Map::new().with_mock(|mock| {
			mock.expect_pressed()
				.returning(|_| Box::new(std::iter::once(_Key::A)));
		}));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));
		app.world_mut().spawn((
			GlobalTransform::from_xyz(1., 2., 3.),
			_Agent(Speed(UnitsPerSecond::new(42.))),
		));

		_ = app.world_mut().run_system_once_with(
			Duration::from_secs(1),
			_Event::trigger_movement::<_Cam, _Agent, _Map, _Key>,
		)?;

		assert_eq!(
			vec![_Event(Vec3::new(1., 2., 3.) + Vec3::Z)],
			move_input_events(&app)
		);
		Ok(())
	}

	#[test]
	fn trigger_accumulated_2_direction_from_center() -> Result<(), RunSystemError> {
		mock_event!();
		let event_ctx = Mock_Event::min_distance_context();
		event_ctx.expect().return_const(1. / SCALE_MIN_DISTANCE);
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
		let mut app = setup::<_Event>(_Map::new().with_mock(|mock| {
			mock.expect_pressed()
				.returning(|_| Box::new([_Key::A, _Key::B].into_iter()));
		}));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));
		app.world_mut().spawn((
			GlobalTransform::default(),
			_Agent(Speed(UnitsPerSecond::new(1.))),
		));

		_ = app.world_mut().run_system_once_with(
			Duration::from_secs(1),
			_Event::trigger_movement::<_Cam, _Agent, _Map, _Key>,
		)?;

		assert_eq!(
			vec![_Event((Vec3::Z + Vec3::X).normalize())],
			move_input_events(&app)
		);
		Ok(())
	}

	#[test]
	fn trigger_accumulated_3_direction_from_center() -> Result<(), RunSystemError> {
		mock_event!();
		let event_ctx = Mock_Event::min_distance_context();
		event_ctx.expect().return_const(1. / SCALE_MIN_DISTANCE);
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
		let mut app = setup::<_Event>(_Map::new().with_mock(|mock| {
			mock.expect_pressed()
				.returning(|_| Box::new([_Key::A, _Key::B, _Key::C].into_iter()));
		}));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));
		app.world_mut().spawn((
			GlobalTransform::default(),
			_Agent(Speed(UnitsPerSecond::new(1.))),
		));

		_ = app.world_mut().run_system_once_with(
			Duration::from_secs(1),
			_Event::trigger_movement::<_Cam, _Agent, _Map, _Key>,
		)?;

		assert_eq!(vec![_Event(Vec3::X)], move_input_events(&app));
		Ok(())
	}

	#[test]
	fn no_trigger_when_accumulated_2_directions_are_zero_from_center() -> Result<(), RunSystemError>
	{
		mock_event!();
		mock_cam!();
		let event_ctx = Mock_Event::min_distance_context();
		event_ctx.expect().return_const(1. / SCALE_MIN_DISTANCE);
		let cam_ctx = Mock_Cam::key_direction_context();
		cam_ctx
			.expect()
			.with(eq(GlobalTransform::default()), eq(_Key::A))
			.returning(|_, _| Ok(Dir3::Z));
		cam_ctx
			.expect()
			.with(eq(GlobalTransform::default()), eq(_Key::B))
			.returning(|_, _| Ok(Dir3::NEG_Z));
		let mut app = setup::<_Event>(_Map::new().with_mock(|mock| {
			mock.expect_pressed()
				.returning(|_| Box::new([_Key::A, _Key::B].into_iter()));
		}));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));
		app.world_mut().spawn((
			GlobalTransform::default(),
			_Agent(Speed(UnitsPerSecond::new(1.))),
		));

		_ = app.world_mut().run_system_once_with(
			Duration::from_secs(1),
			_Event::trigger_movement::<_Cam, _Agent, _Map, _Key>,
		)?;

		assert_eq!(vec![] as Vec<_Event>, move_input_events(&app));
		Ok(())
	}

	#[test]
	fn use_camera_transform() -> Result<(), RunSystemError> {
		mock_event!();
		let event_ctx = Mock_Event::min_distance_context();
		event_ctx.expect().return_const(1. / SCALE_MIN_DISTANCE);
		mock_cam!();
		let cam_ctx = Mock_Cam::key_direction_context();
		cam_ctx
			.expect()
			.with(eq(GlobalTransform::from_xyz(1., 2., 3.)), eq(_Key::A))
			.returning(|_, _| Ok(Dir3::Z));
		let mut app = setup::<_Event>(_Map::new().with_mock(|mock| {
			mock.expect_pressed()
				.returning(move |_| Box::new(std::iter::once(_Key::A)));
		}));
		app.world_mut()
			.spawn((_Cam, GlobalTransform::from_xyz(1., 2., 3.)));
		app.world_mut().spawn((
			GlobalTransform::default(),
			_Agent(Speed(UnitsPerSecond::new(1.))),
		));

		_ = app.world_mut().run_system_once_with(
			Duration::from_secs(1),
			_Event::trigger_movement::<_Cam, _Agent, _Map, _Key>,
		)?;
		Ok(())
	}

	#[test]
	fn use_button_input() -> Result<(), RunSystemError> {
		mock_event!();
		let event_ctx = Mock_Event::min_distance_context();
		event_ctx.expect().return_const(1. / SCALE_MIN_DISTANCE);
		mock_cam!();
		let cam_ctx = Mock_Cam::key_direction_context();
		cam_ctx.expect().returning(|_, _| Ok(Dir3::Z));
		let mut app = setup::<_Event>(_Map::new().with_mock(|mock| {
			mock.expect_pressed().returning(move |input| {
				assert_eq!(
					HashSet::from([KeyCode::Katakana, KeyCode::Hiragana]),
					input.get_just_pressed().copied().collect::<HashSet<_>>()
				);
				Box::new(std::iter::empty())
			});
		}));
		let mut input = ButtonInput::default();
		input.press(KeyCode::Hiragana);
		input.press(KeyCode::Katakana);
		app.insert_resource(input);
		app.world_mut().spawn((_Cam, GlobalTransform::default()));
		app.world_mut().spawn((
			GlobalTransform::from_xyz(1., 2., 3.),
			_Agent(Speed(UnitsPerSecond::new(1.))),
		));

		_ = app.world_mut().run_system_once_with(
			Duration::from_secs(1),
			_Event::trigger_movement::<_Cam, _Agent, _Map, _Key>,
		)?;
		Ok(())
	}

	#[test]
	fn return_no_agent_error() -> Result<(), RunSystemError> {
		mock_event!();
		let event_ctx = Mock_Event::min_distance_context();
		event_ctx.expect().return_const(1. / SCALE_MIN_DISTANCE);
		mock_cam!();
		let mut app = setup::<_Event>(_Map::new().with_mock(|mock| {
			mock.expect_pressed()
				.returning(|_| Box::new(std::iter::empty()));
		}));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));

		let result = app.world_mut().run_system_once_with(
			Duration::from_secs(1),
			_Event::trigger_movement::<_Cam, _Agent, _Map, _Key>,
		)?;

		assert_eq!(Err(TriggerMovementError::from(QueryError::NoAgent)), result);
		Ok(())
	}

	#[test]
	fn return_multiple_agents_error() -> Result<(), RunSystemError> {
		mock_event!();
		let event_ctx = Mock_Event::min_distance_context();
		event_ctx.expect().return_const(1. / SCALE_MIN_DISTANCE);
		mock_cam!();
		let mut app = setup::<_Event>(_Map::new().with_mock(|mock| {
			mock.expect_pressed()
				.returning(|_| Box::new(std::iter::empty()));
		}));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));
		app.world_mut().spawn((
			GlobalTransform::default(),
			_Agent(Speed(UnitsPerSecond::new(1.))),
		));
		app.world_mut().spawn((
			GlobalTransform::default(),
			_Agent(Speed(UnitsPerSecond::new(1.))),
		));

		let result = app.world_mut().run_system_once_with(
			Duration::from_secs(1),
			_Event::trigger_movement::<_Cam, _Agent, _Map, _Key>,
		)?;

		assert_eq!(
			Err(TriggerMovementError::from(QueryError::MultipleAgents)),
			result
		);
		Ok(())
	}

	#[test]
	fn return_no_cam_error() -> Result<(), RunSystemError> {
		mock_event!();
		let event_ctx = Mock_Event::min_distance_context();
		event_ctx.expect().return_const(1. / SCALE_MIN_DISTANCE);
		mock_cam!();
		let mut app = setup::<_Event>(_Map::new().with_mock(|mock| {
			mock.expect_pressed()
				.returning(|_| Box::new(std::iter::empty()));
		}));
		app.world_mut().spawn((
			GlobalTransform::default(),
			_Agent(Speed(UnitsPerSecond::new(1.))),
		));

		let result = app.world_mut().run_system_once_with(
			Duration::from_secs(1),
			_Event::trigger_movement::<_Cam, _Agent, _Map, _Key>,
		)?;

		assert_eq!(Err(TriggerMovementError::from(QueryError::NoCam)), result);
		Ok(())
	}

	#[test]
	fn return_multiple_cams_error() -> Result<(), RunSystemError> {
		mock_event!();
		let event_ctx = Mock_Event::min_distance_context();
		event_ctx.expect().return_const(1. / SCALE_MIN_DISTANCE);
		mock_cam!();
		let mut app = setup::<_Event>(_Map::new().with_mock(|mock| {
			mock.expect_pressed()
				.returning(|_| Box::new(std::iter::empty()));
		}));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));
		app.world_mut().spawn((_Cam, GlobalTransform::default()));
		app.world_mut().spawn((
			GlobalTransform::default(),
			_Agent(Speed(UnitsPerSecond::new(1.))),
		));

		let result = app.world_mut().run_system_once_with(
			Duration::from_secs(1),
			_Event::trigger_movement::<_Cam, _Agent, _Map, _Key>,
		)?;

		assert_eq!(
			Err(TriggerMovementError::from(QueryError::MultipleCams)),
			result
		);
		Ok(())
	}
}
