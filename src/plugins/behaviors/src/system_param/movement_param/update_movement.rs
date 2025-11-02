use crate::system_param::movement_param::MovementContextMut;
use common::{
	tools::UnitsPerSecond,
	traits::{animation::Animation, handles_movement::UpdateMovement},
};

impl<TMotion> UpdateMovement for MovementContextMut<'_, TMotion> {
	fn update(&mut self, speed: UnitsPerSecond, animation: Option<Animation>) {
		let Some(movement_definition) = self.movement_definition.as_deref_mut() else {
			return;
		};

		movement_definition.speed = speed;
		movement_definition.animation = animation;
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
		tools::Units,
		traits::{
			accessors::get::EntityContextMut,
			animation::{AnimationAsset, PlayMode},
			handles_movement::Movement,
		},
	};
	use testing::SingleThreadedApp;

	struct _Motion;

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
			.run_system_once(move |mut p: MovementParamMut<_Motion>| {
				let mut ctx =
					MovementParamMut::get_entity_context_mut(&mut p, entity, Movement).unwrap();
				ctx.update(
					UnitsPerSecond::from(110.),
					Some(Animation {
						asset: AnimationAsset::from("my/other/animation/path"),
						play_mode: PlayMode::Repeat,
					}),
				);
			})?;

		assert_eq!(
			Some(&MovementDefinition {
				radius: Units::from(42.),
				speed: UnitsPerSecond::from(110.),
				animation: Some(Animation {
					asset: AnimationAsset::from("my/other/animation/path"),
					play_mode: PlayMode::Repeat,
				}),
			}),
			app.world().entity(entity).get::<MovementDefinition>(),
		);
		Ok(())
	}
}
