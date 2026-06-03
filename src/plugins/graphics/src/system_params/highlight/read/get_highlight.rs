use crate::{
	components::{camera_labels::OutlinePass, pass_layer::PassLayers},
	system_params::highlight::HighlightContext,
};
use common::traits::handles_graphics::{GetHighlight, Highlight};

impl GetHighlight for HighlightContext<'_> {
	fn get_highlight(&self) -> Highlight {
		let outlined = self
			.pass_layers
			.contains_all(&PassLayers::from(OutlinePass));

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
			camera_labels::{OutlinePass, SecondPass},
			pass_layer::PassLayers,
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
		let entity = app.world_mut().spawn(PassLayers::from(SecondPass)).id();

		let highlight = app.world_mut().run_system_once(move |h: HighlightParam| {
			HighlightParam::try_get_context(&h, Visual { entity }).map(|c| c.get_highlight())
		})?;

		assert_eq!(Some(Highlight::None), highlight);
		Ok(())
	}

	#[test]
	fn highlight() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn(PassLayers::from(OutlinePass)).id();

		let highlight = app.world_mut().run_system_once(move |h: HighlightParam| {
			HighlightParam::try_get_context(&h, Visual { entity }).map(|c| c.get_highlight())
		})?;

		assert_eq!(Some(Highlight::Interacting), highlight);
		Ok(())
	}
}
