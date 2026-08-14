use crate::system_params::interactive_param::InteractiveContextMut;
use common::prelude::*;

impl View<InteractiveState> for InteractiveContextMut<'_> {
	fn view(&self) -> InteractiveState {
		self.state
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{interactive::Interactive, interactive_state::IsActive},
		system_params::interactive_param::InteractiveParamMut,
	};
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::traits::handles_interactive::Interactive as InteractiveKey;
	use test_case::test_case;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test_case(IsActive, InteractiveState::Active;  "active")]
	#[test_case((), InteractiveState::Inactive;  "inactive")]
	fn get_door(
		state: impl Bundle,
		interactive_state: InteractiveState,
	) -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Interactive {
					interactive_type: InteractiveType::Door,
				},
				state,
			))
			.id();

		let got = app
			.world_mut()
			.run_system_once(move |mut i: InteractiveParamMut| {
				InteractiveParamMut::try_get_context_mut(&mut i, InteractiveKey { entity })
					.map(|c| c.view_of::<InteractiveState>())
			})?;

		assert_eq!(Some(interactive_state), got);
		Ok(())
	}
}
