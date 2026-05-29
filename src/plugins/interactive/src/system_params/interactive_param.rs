mod view_interactive_type;

use crate::components::interactive::Interactive;
use bevy::{
	ecs::system::{SystemParam, SystemParamItem},
	prelude::*,
};
use common::traits::{
	accessors::get::{ContextChanged, TryGetContext},
	handles_interactive::Interactive as InteractiveKey,
};

#[derive(SystemParam)]
pub struct InteractiveParam<'w, 's> {
	interactive_entities: Query<'w, 's, Ref<'static, Interactive>>,
}

impl TryGetContext<InteractiveKey> for InteractiveParam<'static, 'static> {
	type TContext<'ctx> = InteractiveContext<'ctx>;

	fn try_get_context<'ctx>(
		param: &'ctx SystemParamItem<Self>,
		InteractiveKey { entity }: InteractiveKey,
	) -> Option<Self::TContext<'ctx>> {
		Some(InteractiveContext {
			interactive: param.interactive_entities.get(entity).ok()?,
		})
	}
}

pub struct InteractiveContext<'ctx> {
	interactive: Ref<'ctx, Interactive>,
}

impl ContextChanged for InteractiveContext<'_> {
	fn context_changed(&self) -> bool {
		self.interactive.is_changed()
	}
}
