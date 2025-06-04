use crate::resources::persistent_entities::{LookupError, PersistentEntities};
use bevy::prelude::*;

impl PersistentEntities {
	pub(crate) fn drain_lookup_errors(
		mut persistent_entities: ResMut<Self>,
	) -> Vec<Result<(), LookupError>> {
		persistent_entities.errors.drain(..).map(Err).collect()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::persistent_entity::PersistentEntity,
		test_tools::utils::SingleThreadedApp,
	};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};

	fn setup(persistent_entities: PersistentEntities) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(persistent_entities);

		app
	}

	#[test]
	fn remove_errors() -> Result<(), RunSystemError> {
		let mut app = setup(PersistentEntities {
			errors: vec![LookupError(PersistentEntity::default())],
			..default()
		});

		app.world_mut()
			.run_system_once(PersistentEntities::drain_lookup_errors)?;

		assert!(
			app.world()
				.resource::<PersistentEntities>()
				.errors
				.is_empty()
		);
		Ok(())
	}

	#[test]
	fn return_errors() -> Result<(), RunSystemError> {
		let error = LookupError(PersistentEntity::default());
		let mut app = setup(PersistentEntities {
			errors: vec![error],
			..default()
		});

		let errors = app
			.world_mut()
			.run_system_once(PersistentEntities::drain_lookup_errors)?;

		assert_eq!(vec![Err(error)], errors);
		Ok(())
	}
}
