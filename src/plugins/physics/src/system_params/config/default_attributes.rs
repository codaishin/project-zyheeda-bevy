use crate::{
	components::default_attributes::DefaultAttributes,
	system_params::config::ConfigContextMut,
};
use common::traits::handles_physics::{ConfigureDefaultAttributes, PhysicalDefaultAttributes};

impl ConfigureDefaultAttributes for ConfigContextMut<'_> {
	fn configure_default_attributes(&mut self, default: PhysicalDefaultAttributes) {
		self.entity.try_insert(DefaultAttributes(default));
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use crate::{
		components::default_attributes::DefaultAttributes,
		system_params::config::ConfigParamMut,
	};

	use super::*;
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::{
		attributes::{effect_target::EffectTarget, health::Health},
		traits::{accessors::get::GetContextMut, handles_physics::NoDefaultAttributes},
	};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn insert_default_attributes() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut p: ConfigParamMut| {
				let key = NoDefaultAttributes { entity };
				let mut ctx = ConfigParamMut::get_context_mut(&mut p, key).unwrap();
				ctx.configure_default_attributes(PhysicalDefaultAttributes {
					health: Health::new(11.),
					force_interaction: EffectTarget::Affected,
					gravity_interaction: EffectTarget::Immune,
				});
			})?;

		assert_eq!(
			Some(&DefaultAttributes(PhysicalDefaultAttributes {
				health: Health::new(11.),
				force_interaction: EffectTarget::Affected,
				gravity_interaction: EffectTarget::Immune,
			})),
			app.world().entity(entity).get::<DefaultAttributes>(),
		);
		Ok(())
	}
}
