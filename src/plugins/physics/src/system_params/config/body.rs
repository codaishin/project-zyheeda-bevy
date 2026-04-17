use crate::{
	components::{center_offset::CenterOffset, physical_body::PhysicalBody},
	system_params::config::ConfigContextMut,
};
use common::{
	tools::Units,
	traits::handles_physics::{ConfigureBody, physical_bodies::Body},
};

impl ConfigureBody for ConfigContextMut<'_> {
	fn configure_body(&mut self, body: Body, center_offset: Units) {
		self.entity
			.try_insert((PhysicalBody(body), CenterOffset(center_offset)));
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{
		components::{center_offset::CenterOffset, physical_body::PhysicalBody},
		system_params::config::ConfigParamMut,
	};
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::{
		tools::Units,
		traits::{
			accessors::get::GetContextMut,
			handles_physics::{NoBodyConfigured, physical_bodies::Shape},
		},
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
				let mut ctx = ConfigParamMut::get_context_mut(&mut p, key).unwrap();
				ctx.configure_body(Body::from_shape(Shape::StaticGltfMesh3d), Units::ZERO);
			})?;

		assert_eq!(
			Some(&PhysicalBody(Body::from_shape(Shape::StaticGltfMesh3d))),
			app.world().entity(entity).get::<PhysicalBody>(),
		);
		Ok(())
	}

	#[test]
	fn insert_offset() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut p: ConfigParamMut| {
				let key = NoBodyConfigured { entity };
				let mut ctx = ConfigParamMut::get_context_mut(&mut p, key).unwrap();
				ctx.configure_body(Body::from_shape(Shape::StaticGltfMesh3d), Units::from(11.));
			})?;

		assert_eq!(
			Some(&CenterOffset(Units::from(11.))),
			app.world().entity(entity).get::<CenterOffset>(),
		);
		Ok(())
	}
}
