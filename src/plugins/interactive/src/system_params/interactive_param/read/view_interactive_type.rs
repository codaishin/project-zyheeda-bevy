use crate::system_params::interactive_param::InteractiveContext;
use common::traits::{accessors::get::View, handles_map_generation::InteractiveType};

impl View<InteractiveType> for InteractiveContext<'_> {
	fn view(&self) -> InteractiveType {
		self.interactive.interactive_type
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::interactive::Interactive,
		system_params::interactive_param::InteractiveParam,
	};
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::traits::{
		accessors::get::{TryGetContext, ViewOf},
		handles_interactive::Interactive as InteractiveKey,
	};
	use test_case::test_case;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test_case(InteractiveType::Door;  "door")]
	#[test_case(InteractiveType::Container;  "container")]
	fn get_door(interactive_type: InteractiveType) -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn(Interactive { interactive_type }).id();

		let got = app
			.world_mut()
			.run_system_once(move |i: InteractiveParam| {
				InteractiveParam::try_get_context(&i, InteractiveKey { entity })
					.map(|c| c.view_of::<InteractiveType>())
			})?;

		assert_eq!(Some(interactive_type), got);
		Ok(())
	}
}
