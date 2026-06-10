use crate::system_params::lights::TorchLightContext;
use common::traits::handles_light::{GetLight, Light, Lumen};

impl GetLight for TorchLightContext<'_> {
	fn get_light(&self) -> Light {
		let intensity = match self.light {
			Some(ref l) => l.intensity,
			None => Lumen::ZERO,
		};

		Light { intensity }
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::torch_light::TorchLight, system_params::lights::Lights};
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::traits::{
		accessors::get::TryGetContext,
		handles_light::TorchLight as TorchLightKey,
	};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn get_light() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(TorchLight {
				intensity: Lumen::from(11.),
			})
			.id();

		let light = app.world_mut().run_system_once(move |l: Lights| {
			Lights::try_get_context(&l, TorchLightKey { entity }).map(|c| c.get_light())
		})?;

		assert_eq!(
			Some(Light {
				intensity: Lumen::from(11.)
			}),
			light
		);
		Ok(())
	}

	#[test]
	fn get_light_when_component_missing() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		let light = app.world_mut().run_system_once(move |l: Lights| {
			Lights::try_get_context(&l, TorchLightKey { entity }).map(|c| c.get_light())
		})?;

		assert_eq!(
			Some(Light {
				intensity: Lumen::ZERO
			}),
			light
		);
		Ok(())
	}
}
