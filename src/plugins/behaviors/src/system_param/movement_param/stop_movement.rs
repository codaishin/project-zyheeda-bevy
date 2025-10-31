use crate::{
	components::movement_definition::MovementDefinition,
	system_param::movement_param::MovementContextMut,
};
use common::traits::handles_movement::StopMovement;

impl StopMovement for MovementContextMut<'_> {
	fn stop(&mut self) {
		self.entity.try_remove::<MovementDefinition>();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::movement_definition::MovementDefinition,
		system_param::movement_param::MovementParamMut,
	};
	use bevy::{
		app::{App, Update},
		ecs::system::{RunSystemError, RunSystemOnce},
	};
	use common::{
		tools::{Units, UnitsPerSecond},
		traits::{
			accessors::get::EntityContextMut,
			animation::{Animation, AnimationAsset, PlayMode},
			handles_movement::Movement,
		},
	};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn insert_movement_definition() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(MovementDefinition {
				radius: Units::from(42.),
				speed: UnitsPerSecond::from(11.),
				animation: Some(Animation {
					asset: AnimationAsset::from("my/animation/path"),
					play_mode: PlayMode::Repeat,
				}),
			})
			.id();

		app.world_mut()
			.run_system_once(move |mut p: MovementParamMut| {
				let mut ctx =
					MovementParamMut::get_entity_context_mut(&mut p, entity, Movement).unwrap();
				ctx.stop();
			})?;

		assert_eq!(None, app.world().entity(entity).get::<MovementDefinition>());
		Ok(())
	}
}
