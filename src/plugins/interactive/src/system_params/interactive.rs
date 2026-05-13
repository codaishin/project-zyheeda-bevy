mod set_interactive_role;

use bevy::{
	ecs::system::{SystemParam, SystemParamItem},
	prelude::*,
};
use common::{
	traits::{
		accessors::get::{GetContextMut, GetMut},
		handles_interactive::SetInteractive,
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};

#[derive(SystemParam)]
pub struct InteractiveMut<'w, 's> {
	commands: ZyheedaCommands<'w, 's>,
}

impl GetContextMut<SetInteractive> for InteractiveMut<'static, 'static> {
	type TContext<'ctx> = InteractiveContextMut<'ctx>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut SystemParamItem<Self>,
		SetInteractive { entity }: SetInteractive,
	) -> Option<Self::TContext<'ctx>> {
		Some(InteractiveContextMut {
			entity: param.commands.get_mut(&entity)?,
		})
	}
}

pub struct InteractiveContextMut<'ctx> {
	entity: ZyheedaEntityCommands<'ctx>,
}
