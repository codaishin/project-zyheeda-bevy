use crate::materials::effect_material::{EffectMaterial, ImpactEffect, NO_IMPACTS};
use bevy::{asset::InvalidGenerationError, prelude::*, render::storage::ShaderBuffer};

impl EffectMaterial {
	pub(crate) fn set_default_impacts(
		mut buffers: ResMut<Assets<ShaderBuffer>>,
	) -> Result<(), InvalidGenerationError> {
		buffers.insert(
			&*NO_IMPACTS,
			ShaderBuffer::from(Vec::<ImpactEffect>::default()),
		)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		render::storage::ShaderBuffer,
	};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<Assets<ShaderBuffer>>();

		app
	}

	#[test]
	fn set_empty() -> Result<(), RunSystemError> {
		let mut app = setup();

		_ = app
			.world_mut()
			.run_system_once(EffectMaterial::set_default_impacts);

		assert_eq!(
			Some(&ShaderBuffer::from(Vec::<ImpactEffect>::default()).data),
			app.world()
				.resource::<Assets<ShaderBuffer>>()
				.get(&*NO_IMPACTS)
				.map(|s| &s.data)
		);
		Ok(())
	}

	#[test]
	fn result_ok() -> Result<(), RunSystemError> {
		let mut app = setup();

		let result = app
			.world_mut()
			.run_system_once(EffectMaterial::set_default_impacts);

		assert!(result.is_ok());
		Ok(())
	}
}
