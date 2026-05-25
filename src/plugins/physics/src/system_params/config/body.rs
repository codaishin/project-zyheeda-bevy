use crate::{
	components::{
		body::Body,
		offset::{AimOffset, CenterOffset},
	},
	system_params::config::ConfigContextMut,
};
use common::traits::handles_physics::{
	ConfigureBody,
	TranslationOffsets,
	physical_bodies::BodyConfig,
};

impl ConfigureBody for ConfigContextMut<'_> {
	fn configure_body(&mut self, body: BodyConfig, offsets: TranslationOffsets) {
		self.entity.try_insert((
			Body(body),
			AimOffset(offsets.aim),
			CenterOffset(offsets.center),
		));
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{
		components::offset::{AimOffset, CenterOffset},
		system_params::config::ConfigParamMut,
	};
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::traits::{
		accessors::get::TryGetContextMut,
		handles_physics::{NoBodyConfigured, physical_bodies::Shape},
	};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn insert_physical_body() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut p: ConfigParamMut| {
				let key = NoBodyConfigured { entity };
				let mut ctx = ConfigParamMut::try_get_context_mut(&mut p, key).unwrap();
				ctx.configure_body(
					BodyConfig::from_shape(Shape::StaticGltfMesh3d),
					TranslationOffsets::ZERO,
				);
			})?;

		assert_eq!(
			Some(&Body(BodyConfig::from_shape(Shape::StaticGltfMesh3d))),
			app.world().entity(entity).get::<Body>(),
		);
		Ok(())
	}

	#[test]
	fn insert_offsets() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut p: ConfigParamMut| {
				let key = NoBodyConfigured { entity };
				let mut ctx = ConfigParamMut::try_get_context_mut(&mut p, key).unwrap();
				ctx.configure_body(
					BodyConfig::from_shape(Shape::StaticGltfMesh3d),
					TranslationOffsets {
						aim: 11.,
						center: 12.,
					},
				);
			})?;

		assert_eq!(
			(Some(&AimOffset(11.)), Some(&CenterOffset(12.)),),
			(
				app.world().entity(entity).get::<AimOffset>(),
				app.world().entity(entity).get::<CenterOffset>(),
			)
		);
		Ok(())
	}
}
