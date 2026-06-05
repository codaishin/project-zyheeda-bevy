mod read;
mod write;

use crate::components::model_render_layers::ModelRenderLayers;
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
	model_render_layers: Query<'w, 's, Ref<'static, ModelRenderLayers>>,
}

impl TryGetContext<Visual> for HighlightParam<'static, 'static> {
	type TContext<'ctx> = HighlightContext<'ctx>;

	fn try_get_context<'ctx>(
		param: &'ctx SystemParamItem<Self>,
		Visual { entity }: Visual,
	) -> Option<Self::TContext<'ctx>> {
		let model_render_layers = param.model_render_layers.get(entity).ok()?;

		Some(HighlightContext {
			model_render_layers,
		})
	}
}

pub struct HighlightContext<'ctx> {
	model_render_layers: Ref<'ctx, ModelRenderLayers>,
}

impl ContextChanged for HighlightContext<'_> {
	fn context_changed(&self) -> bool {
		self.model_render_layers.is_changed()
	}
}

#[derive(SystemParam)]
pub struct HighlightParamMut<'w, 's> {
	model_render_layers: Query<'w, 's, &'static mut ModelRenderLayers>,
}

impl TryGetContextMut<Visual> for HighlightParamMut<'static, 'static> {
	type TContext<'ctx> = HighlightContextMut<'ctx>;

	fn try_get_context_mut<'ctx>(
		param: &'ctx mut SystemParamItem<Self>,
		Visual { entity }: Visual,
	) -> Option<Self::TContext<'ctx>> {
		let pass_layers = param.model_render_layers.get_mut(entity).ok()?;

		Some(HighlightContextMut {
			model_render_layers: pass_layers,
		})
	}
}

pub struct HighlightContextMut<'ctx> {
	model_render_layers: Mut<'ctx, ModelRenderLayers>,
}
