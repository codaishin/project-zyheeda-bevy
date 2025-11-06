mod override_face;
mod register_face_target_definition;

use bevy::{ecs::system::SystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{GetContextMut, GetMut},
		handles_orientation::Facing,
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};

#[derive(SystemParam)]
pub struct FaceParamMut<'w, 's> {
	commands: ZyheedaCommands<'w, 's>,
}

impl GetContextMut<Facing> for FaceParamMut<'_, '_> {
	type TContext<'ctx> = FaceContextMut<'ctx>;

	fn get_context_mut<'ctx>(
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
