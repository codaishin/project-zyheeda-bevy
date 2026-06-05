use crate::{
	components::{camera_labels::OutlinePass, model_render_layers::ModelRenderLayers},
	system_params::highlight::HighlightContextMut,
};
use common::traits::handles_graphics::{Highlight, SetHighlight};

impl SetHighlight for HighlightContextMut<'_> {
	fn set_highlight(&mut self, highlight: Highlight) {
		match highlight {
			Highlight::None => self.model_render_layers.reset(),
			Highlight::Interacting => self
				.model_render_layers
				.add_layers(ModelRenderLayers::from(OutlinePass)),
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{
		components::{
			camera_labels::{OutlinePass, SecondPass},
			model_render_layers::ModelRenderLayers,
		},
		system_params::highlight::HighlightParamMut,
	};
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::traits::{accessors::get::TryGetContextMut, handles_graphics::Visual};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn set_highlight() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(ModelRenderLayers::from(SecondPass))
			.id();

		app.world_mut()
			.run_system_once(move |mut h: HighlightParamMut| {
				let mut ctx =
					HighlightParamMut::try_get_context_mut(&mut h, Visual { entity }).unwrap();
				ctx.set_highlight(Highlight::Interacting);
			})?;

		let mut expected = ModelRenderLayers::from(SecondPass);
		expected.add_layers(ModelRenderLayers::from(OutlinePass));
		assert_eq!(
			Some(&expected),
			app.world().entity(entity).get::<ModelRenderLayers>()
		);
		Ok(())
	}

	#[test]
	fn remove_highlight() -> Result<(), RunSystemError> {
		let mut app = setup();
		let mut layers = ModelRenderLayers::from(SecondPass);
		layers.add_layers(ModelRenderLayers::from(OutlinePass));
		let entity = app.world_mut().spawn(layers).id();

		app.world_mut()
			.run_system_once(move |mut h: HighlightParamMut| {
				let mut ctx =
					HighlightParamMut::try_get_context_mut(&mut h, Visual { entity }).unwrap();
				ctx.set_highlight(Highlight::None);
			})?;

		assert_eq!(
			Some(&ModelRenderLayers::from(SecondPass)),
			app.world().entity(entity).get::<ModelRenderLayers>()
		);
		Ok(())
	}
}
