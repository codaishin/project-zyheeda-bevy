use crate::{
	components::{camera_labels::OutlinePass, model_render_layers::ModelRenderLayers},
	system_params::highlight::HighlightContext,
};
use common::traits::handles_graphics::{GetHighlight, Highlight};

impl GetHighlight for HighlightContext<'_> {
	fn get_highlight(&self) -> Highlight {
		let outlined = self
			.model_render_layers
			.contains_all(&ModelRenderLayers::from(OutlinePass));

		if outlined {
			Highlight::Interacting
		} else {
			Highlight::None
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{
			camera_labels::{OutlinePass, CompositePass},
			model_render_layers::ModelRenderLayers,
		},
		system_params::highlight::HighlightParam,
	};
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::traits::{accessors::get::TryGetContext, handles_graphics::Visual};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn no_highlight() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(ModelRenderLayers::from(CompositePass))
			.id();

		let highlight = app.world_mut().run_system_once(move |h: HighlightParam| {
			HighlightParam::try_get_context(&h, Visual { entity }).map(|c| c.get_highlight())
		})?;

		assert_eq!(Some(Highlight::None), highlight);
		Ok(())
	}

	#[test]
	fn highlight() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(ModelRenderLayers::from(OutlinePass))
			.id();

		let highlight = app.world_mut().run_system_once(move |h: HighlightParam| {
			HighlightParam::try_get_context(&h, Visual { entity }).map(|c| c.get_highlight())
		})?;

		assert_eq!(Some(Highlight::Interacting), highlight);
		Ok(())
	}
}
