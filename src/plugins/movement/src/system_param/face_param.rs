mod override_face;

use bevy::{ecs::system::SystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{GetMut, TryGetContextMut},
		handles_orientation::Facing,
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};

#[derive(SystemParam)]
pub struct FaceParamMut<'w, 's> {
	commands: ZyheedaCommands<'w, 's>,
}

impl TryGetContextMut<Facing> for FaceParamMut<'static, 'static> {
	type TContext<'ctx> = FaceContextMut<'ctx>;

	fn try_get_context_mut<'ctx>(
		param: &'ctx mut FaceParamMut,
		Facing { entity }: Facing,
	) -> Option<Self::TContext<'ctx>> {
		let entity = param.commands.get_mut(&entity)?;

		Some(FaceContextMut { entity })
	}
}

pub struct FaceContextMut<'ctx> {
	entity: ZyheedaEntityCommands<'ctx>,
}
