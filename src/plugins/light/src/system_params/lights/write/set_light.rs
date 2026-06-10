use crate::{components::torch_light::TorchLight, system_params::lights::TorchLightContextMut};
use common::traits::handles_light::{Light, SetLight};

impl SetLight for TorchLightContextMut<'_> {
	fn set_light(&mut self, Light { intensity }: Light) {
		self.entity.try_insert(TorchLight { intensity });
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::system_params::lights::LightsMut;
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::{
		tools::Units,
		traits::{accessors::get::TryGetContextMut, handles_light::TorchLight as TorchLightKey},
	};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn set_light() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut().run_system_once(move |mut l: LightsMut| {
			LightsMut::try_get_context_mut(&mut l, TorchLightKey { entity }).map(|mut c| {
				c.set_light(Light {
					intensity: Units::from(42.),
				})
			})
		})?;

		assert_eq!(
			Some(&TorchLight {
				intensity: Units::from(42.)
			}),
			app.world().entity(entity).get::<TorchLight>(),
		);
		Ok(())
	}
}
