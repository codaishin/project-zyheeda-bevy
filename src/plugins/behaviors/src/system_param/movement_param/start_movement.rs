use crate::{
	components::movement_definition::MovementDefinition,
	system_param::movement_param::MovementContextMut,
};
use common::{
	tools::{Units, UnitsPerSecond},
	traits::{animation::Animation, handles_movement::StartMovement},
};

impl StartMovement for MovementContextMut<'_> {
	fn start(&mut self, radius: Units, speed: UnitsPerSecond, animation: Option<Animation>) {
		self.entity.try_insert(MovementDefinition {
			radius,
			speed,
			animation,
		});
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
	use common::traits::{
		accessors::get::EntityContextMut,
		animation::{AnimationAsset, PlayMode},
		handles_movement::Movement,
	};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn insert_movement_definition() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut p: MovementParamMut| {
				let mut ctx =
					MovementParamMut::get_entity_context_mut(&mut p, entity, Movement).unwrap();
				ctx.start(
					Units::from(42.),
					UnitsPerSecond::from(11.),
					Some(Animation {
						asset: AnimationAsset::from("my/animation/path"),
						play_mode: PlayMode::Repeat,
					}),
				);
			})?;

		assert_eq!(
			Some(&MovementDefinition {
				radius: Units::from(42.),
				speed: UnitsPerSecond::from(11.),
				animation: Some(Animation {
					asset: AnimationAsset::from("my/animation/path"),
					play_mode: PlayMode::Repeat,
				}),
			}),
			app.world().entity(entity).get::<MovementDefinition>(),
		);
		Ok(())
	}
}
