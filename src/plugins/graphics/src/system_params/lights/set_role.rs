use crate::{
	components::roles::{Enemy, Player},
	system_params::lights::RolesContextMut,
};
use common::traits::handles_graphics::{Role, SetRole};

impl SetRole for RolesContextMut<'_> {
	fn set_role(&mut self, role: Role) {
		match role {
			Role::Player => self.entity.try_insert(Player),
			Role::Enemy => self.entity.try_insert(Enemy),
		};
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::roles::{Enemy, Player},
		system_params::lights::RolesParamMut,
	};
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::traits::{accessors::get::TryGetContextMut, handles_graphics::HasNoRole};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn set_player() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut l: RolesParamMut| {
				RolesParamMut::try_get_context_mut(&mut l, HasNoRole { entity })
					.map(|mut c| c.set_role(Role::Player))
			})?;

		assert_eq!(Some(&Player), app.world().entity(entity).get::<Player>(),);
		Ok(())
	}

	#[test]
	fn set_enemy() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut l: RolesParamMut| {
				RolesParamMut::try_get_context_mut(&mut l, HasNoRole { entity })
					.map(|mut c| c.set_role(Role::Enemy))
			})?;

		assert_eq!(Some(&Enemy), app.world().entity(entity).get::<Enemy>(),);
		Ok(())
	}
}
