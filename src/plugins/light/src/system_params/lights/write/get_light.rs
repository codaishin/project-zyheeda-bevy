use crate::system_params::lights::TorchLightContextMut;
use common::{
	tools::Units,
	traits::handles_light::{GetLight, Light},
};

impl GetLight for TorchLightContextMut<'_> {
	fn get_light(&self) -> Light {
		let intensity = match self.light {
			Some(l) => l.intensity,
			None => Units::ZERO,
		};

		Light { intensity }
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::torch_light::TorchLight, system_params::lights::LightsMut};
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
	fn get_light() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(TorchLight {
				intensity: Units::from(11.),
			})
			.id();

		let light = app.world_mut().run_system_once(move |mut l: LightsMut| {
			LightsMut::try_get_context_mut(&mut l, TorchLightKey { entity }).map(|c| c.get_light())
		})?;

		assert_eq!(
			Some(Light {
				intensity: Units::from(11.)
			}),
			light
		);
		Ok(())
	}

	#[test]
	fn get_light_when_component_missing() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		let light = app.world_mut().run_system_once(move |mut l: LightsMut| {
			LightsMut::try_get_context_mut(&mut l, TorchLightKey { entity }).map(|c| c.get_light())
		})?;

		assert_eq!(
			Some(Light {
				intensity: Units::ZERO
			}),
			light
		);
		Ok(())
	}
}
