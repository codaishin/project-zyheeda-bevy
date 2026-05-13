use crate::{
	components::interactive_role::InteractiveRole,
	system_params::interactive::InteractiveContextMut,
};
use common::traits::handles_interactive::{Interactive, SetInteractiveRole};

impl SetInteractiveRole for InteractiveContextMut<'_> {
	fn set_interactive_role(&mut self, role: Interactive) {
		self.entity.try_insert(InteractiveRole(role));
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{
		components::interactive_role::InteractiveRole,
		system_params::interactive::InteractiveMut,
	};
	use bevy::{
		app::App,
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::traits::{
		accessors::get::GetContextMut,
		handles_interactive::{Door, SetInteractive},
	};
	use test_case::test_case;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test_case(Interactive::Door(Door::SlideDoor); "slide door")]
	fn set_interactive(interactive: Interactive) -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut p: InteractiveMut| {
				InteractiveMut::get_context_mut(&mut p, SetInteractive { entity })
					.unwrap()
					.set_interactive_role(interactive);
			})?;

		assert_eq!(
			Some(&InteractiveRole(interactive)),
			app.world().entity(entity).get::<InteractiveRole>()
		);
		Ok(())
	}
}
