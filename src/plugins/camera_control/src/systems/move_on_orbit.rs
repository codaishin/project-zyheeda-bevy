use crate::traits::orbit::MoveArm;
use bevy::{
	ecs::{
		component::Mutable,
		system::{StaticSystemParam, SystemParam},
	},
	input::mouse::MouseMotion,
	prelude::*,
};
use common::{
	tools::action_key::camera_key::CameraKey,
	traits::handles_input::{GetInputState, InputState},
};

impl<T> MoveArmsSystem for T where T: Component<Mutability = Mutable> + MoveArm {}

pub(crate) trait MoveArmsSystem: Component<Mutability = Mutable> + MoveArm + Sized {
	fn move_arms<TInput>(
		input: StaticSystemParam<TInput>,
		mut arms: Query<&mut Self>,
		mut mouse_motion: MessageReader<MouseMotion>,
	) where
		TInput: for<'w, 's> SystemParam<Item<'w, 's>: GetInputState>,
	{
		let InputState::Pressed { .. } = input.get_input_state(CameraKey::Rotate) else {
			return;
		};

		for event in mouse_motion.read() {
			for mut arm in &mut arms {
				arm.move_arm(event.delta);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::orbit::Vec2Radians;
	use bevy::input::mouse::MouseMotion;
	use common::{CommonPlugin, tools::action_key::ActionKey};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use test_case::test_case;
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Component, NestedMocks)]
	struct _Arm {
		mock: Mock_Arm,
	}

	#[automock]
	impl MoveArm for _Arm {
		fn move_arm(&mut self, angles: Vec2Radians) {
			self.mock.move_arm(angles);
		}
	}

	#[derive(Resource, NestedMocks)]
	struct _Input {
		mock: Mock_Input,
	}

	#[automock]
	impl GetInputState for _Input {
		fn get_input_state<TAction>(&self, action: TAction) -> InputState
		where
			TAction: Into<ActionKey> + 'static,
		{
			self.mock.get_input_state(action)
		}
	}

	fn setup(map: _Input) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(CommonPlugin::with_asset_loading(false));
		app.add_systems(Update, _Arm::move_arms::<Res<_Input>>);
		app.add_message::<MouseMotion>();

		app.insert_resource(map);

		app
	}

	#[test_case(InputState::pressed(); "pressed")]
	#[test_case(InputState::just_pressed(); "just pressed")]
	fn move_camera_on(state: InputState) {
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_input_state()
				.with(eq(CameraKey::Rotate))
				.return_const(state);
		}));
		app.world_mut().spawn(_Arm::new().with_mock(|mock| {
			mock.expect_move_arm()
				.with(eq(Vec2::new(3., 4.)))
				.once()
				.return_const(());
		}));
		app.world_mut()
			.resource_mut::<Messages<MouseMotion>>()
			.write(MouseMotion {
				delta: Vec2::new(3., 4.),
			});

		app.update();
	}

	#[test_case(InputState::released(); "released")]
	#[test_case(InputState::just_released(); "just released")]
	fn orbit_zero_angle_on(state: InputState) {
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_input_state()
				.with(eq(CameraKey::Rotate))
				.return_const(state);
		}));
		app.world_mut().spawn(_Arm::new().with_mock(|mock| {
			mock.expect_move_arm().never().return_const(());
		}));
		app.world_mut()
			.resource_mut::<Messages<MouseMotion>>()
			.write(MouseMotion {
				delta: Vec2::new(3., 4.),
			});

		app.update();
	}
}
