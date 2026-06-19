mod set_role;

use crate::components::roles::RoleAssigned;
use bevy::{
	ecs::system::{SystemParam, SystemParamItem},
	prelude::*,
};
use common::{
	traits::{
		accessors::get::{GetMut, TryGetContextMut},
		handles_graphics::HasNoRole,
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};

#[derive(SystemParam)]
pub struct RolesParamMut<'w, 's> {
	commands: ZyheedaCommands<'w, 's>,
	roles: Query<'w, 's, (), With<RoleAssigned>>,
}

impl TryGetContextMut<HasNoRole> for RolesParamMut<'static, 'static> {
	type TContext<'ctx> = RolesContextMut<'ctx>;

	fn try_get_context_mut<'ctx>(
		param: &'ctx mut SystemParamItem<Self>,
		HasNoRole { entity }: HasNoRole,
	) -> Option<Self::TContext<'ctx>> {
		if param.roles.contains(entity) {
			return None;
		}

		let entity = param.commands.get_mut(&entity)?;

		Some(RolesContextMut { entity })
	}
}

pub struct RolesContextMut<'ctx> {
	entity: ZyheedaEntityCommands<'ctx>,
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::roles::{Enemy, Player};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use test_case::test_case;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test_case(Player; "player")]
	#[test_case(Enemy; "enemy")]
	fn no_context_when_roles_set_to(role: impl Component) -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn(role).id();

		let ctx = app
			.world_mut()
			.run_system_once(move |mut r: RolesParamMut| {
				RolesParamMut::try_get_context_mut(&mut r, HasNoRole { entity }).is_some()
			})?;

		assert!(!ctx);
		Ok(())
	}
}
