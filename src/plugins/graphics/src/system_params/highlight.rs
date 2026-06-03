mod read;
mod write;

use crate::components::pass_layer::PassLayers;
use bevy::{
	ecs::system::{SystemParam, SystemParamItem},
	prelude::*,
};
use common::traits::{
	accessors::get::{ContextChanged, TryGetContext, TryGetContextMut},
	handles_graphics::Visual,
};

#[derive(SystemParam)]
pub struct HighlightParam<'w, 's> {
	layers: Query<'w, 's, Ref<'static, PassLayers>>,
}

impl TryGetContext<Visual> for HighlightParam<'static, 'static> {
	type TContext<'ctx> = HighlightContext<'ctx>;

	fn try_get_context<'ctx>(
		param: &'ctx SystemParamItem<Self>,
		Visual { entity }: Visual,
	) -> Option<Self::TContext<'ctx>> {
		let pass_layers = param.layers.get(entity).ok()?;

		Some(HighlightContext { pass_layers })
	}
}

pub struct HighlightContext<'ctx> {
	pass_layers: Ref<'ctx, PassLayers>,
}

impl ContextChanged for HighlightContext<'_> {
	fn context_changed(&self) -> bool {
		self.pass_layers.is_changed()
	}
}

#[derive(SystemParam)]
pub struct HighlightParamMut<'w, 's> {
	entities: Query<'w, 's, &'static mut PassLayers>,
}

impl TryGetContextMut<Visual> for HighlightParamMut<'static, 'static> {
	type TContext<'ctx> = HighlightContextMut<'ctx>;

	fn try_get_context_mut<'ctx>(
		param: &'ctx mut SystemParamItem<Self>,
		Visual { entity }: Visual,
	) -> Option<Self::TContext<'ctx>> {
		let pass_layers = param.entities.get_mut(entity).ok()?;

		Some(HighlightContextMut { pass_layers })
	}
}

pub struct HighlightContextMut<'ctx> {
	pass_layers: Mut<'ctx, PassLayers>,
}
