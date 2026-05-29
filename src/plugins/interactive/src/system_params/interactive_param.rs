mod read;
mod write;

use crate::components::{interactive::Interactive, interactive_state::IsActive};
use bevy::{
	ecs::system::{SystemParam, SystemParamItem},
	prelude::*,
};
use common::{
	traits::{
		accessors::get::{ContextChanged, GetMut, TryGetContext, TryGetContextMut},
		handles_interactive::{Interactive as InteractiveKey, InteractiveState},
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};

#[derive(SystemParam)]
pub struct InteractiveParam<'w, 's> {
	interactive_entities: Query<'w, 's, Ref<'static, Interactive>>,
	actives: Query<'w, 's, (), With<IsActive>>,
}

impl TryGetContext<InteractiveKey> for InteractiveParam<'static, 'static> {
	type TContext<'ctx> = InteractiveContext<'ctx>;

	fn try_get_context<'ctx>(
		param: &'ctx SystemParamItem<Self>,
		InteractiveKey { entity }: InteractiveKey,
	) -> Option<Self::TContext<'ctx>> {
		Some(InteractiveContext {
			interactive: param.interactive_entities.get(entity).ok()?,
			state: match param.actives.contains(entity) {
				true => InteractiveState::Active,
				false => InteractiveState::Inactive,
			},
		})
	}
}

pub struct InteractiveContext<'ctx> {
	interactive: Ref<'ctx, Interactive>,
	state: InteractiveState,
}

impl ContextChanged for InteractiveContext<'_> {
	fn context_changed(&self) -> bool {
		self.interactive.is_changed()
	}
}

#[derive(SystemParam)]
pub struct InteractiveParamMut<'w, 's> {
	commands: ZyheedaCommands<'w, 's>,
	interactive_entities: Query<'w, 's, &'static Interactive>,
	actives: Query<'w, 's, (), With<IsActive>>,
}

impl TryGetContextMut<InteractiveKey> for InteractiveParamMut<'static, 'static> {
	type TContext<'ctx> = InteractiveContextMut<'ctx>;

	fn try_get_context_mut<'ctx>(
		param: &'ctx mut SystemParamItem<Self>,
		InteractiveKey { entity }: InteractiveKey,
	) -> Option<Self::TContext<'ctx>> {
		Some(InteractiveContextMut {
			interactive: param.interactive_entities.get(entity).ok()?,
			entity: param.commands.get_mut(&entity)?,
			state: match param.actives.contains(entity) {
				true => InteractiveState::Active,
				false => InteractiveState::Inactive,
			},
		})
	}
}

pub struct InteractiveContextMut<'ctx> {
	entity: ZyheedaEntityCommands<'ctx>,
	interactive: &'ctx Interactive,
	state: InteractiveState,
}
