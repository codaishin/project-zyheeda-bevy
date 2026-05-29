use crate::{
	components::interactive_state::IsActive,
	system_params::interactive_param::InteractiveContextMut,
};
use common::traits::handles_interactive::{InteractiveState, SetInteractiveState};

impl SetInteractiveState for InteractiveContextMut<'_> {
	fn set_interactive_state(&mut self, interactive_state: InteractiveState) {
		match interactive_state {
			InteractiveState::Active => self.entity.try_insert(IsActive),
			InteractiveState::Inactive => self.entity.try_remove::<IsActive>(),
		};
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
	use common::traits::{
		accessors::get::TryGetContextMut,
		handles_interactive::Interactive as InteractiveKey,
		handles_map_generation::InteractiveType,
	};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	fn set(entity: Entity, interactive_state: InteractiveState) -> impl FnMut(InteractiveParamMut) {
		move |mut i: InteractiveParamMut| {
			let key = InteractiveKey { entity };
			let Some(mut ctx) = InteractiveParamMut::try_get_context_mut(&mut i, key) else {
				return;
			};

			ctx.set_interactive_state(interactive_state);
		}
	}

	#[test]
	fn set_active() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Interactive {
				interactive_type: InteractiveType::Door,
			})
			.id();

		app.world_mut()
			.run_system_once(set(entity, InteractiveState::Active))?;

		assert!(app.world().entity(entity).contains::<IsActive>());
		Ok(())
	}

	#[test]
	fn set_inactive() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Interactive {
					interactive_type: InteractiveType::Door,
				},
				IsActive,
			))
			.id();

		app.world_mut()
			.run_system_once(set(entity, InteractiveState::Inactive))?;

		assert!(!app.world().entity(entity).contains::<IsActive>());
		Ok(())
	}
}
