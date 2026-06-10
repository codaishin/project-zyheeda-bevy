mod read;
mod write;

use crate::components::torch_light::TorchLight;
use bevy::{
	ecs::system::{SystemParam, SystemParamItem},
	prelude::*,
};
use common::{
	traits::{
		accessors::get::{ContextChanged, GetMut, TryGetContext, TryGetContextMut},
		handles_light::TorchLight as TorchLightKey,
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};

#[derive(SystemParam)]
pub struct Lights<'w, 's> {
	lights: Query<'w, 's, Ref<'static, TorchLight>>,
}

impl TryGetContext<TorchLightKey> for Lights<'static, 'static> {
	type TContext<'ctx> = TorchLightContext<'ctx>;

	fn try_get_context<'ctx>(
		param: &'ctx SystemParamItem<Self>,
		TorchLightKey { entity }: TorchLightKey,
	) -> Option<Self::TContext<'ctx>> {
		let light = param.lights.get(entity).ok();

		Some(TorchLightContext { light })
	}
}

pub struct TorchLightContext<'ctx> {
	light: Option<Ref<'ctx, TorchLight>>,
}

impl ContextChanged for TorchLightContext<'_> {
	fn context_changed(&self) -> bool {
		self.light.as_ref().is_some_and(Ref::is_changed)
	}
}

#[derive(SystemParam)]
pub struct LightsMut<'w, 's> {
	commands: ZyheedaCommands<'w, 's>,
	lights: Query<'w, 's, &'static TorchLight>,
}

impl TryGetContextMut<TorchLightKey> for LightsMut<'static, 'static> {
	type TContext<'ctx> = TorchLightContextMut<'ctx>;

	fn try_get_context_mut<'ctx>(
		param: &'ctx mut SystemParamItem<Self>,
		TorchLightKey { entity }: TorchLightKey,
	) -> Option<Self::TContext<'ctx>> {
		let light = param.lights.get(entity).ok();
		let entity = param.commands.get_mut(&entity)?;

		Some(TorchLightContextMut { entity, light })
	}
}

pub struct TorchLightContextMut<'ctx> {
	entity: ZyheedaEntityCommands<'ctx>,
	light: Option<&'ctx TorchLight>,
}
