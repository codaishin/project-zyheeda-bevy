use crate::{
	components::movement_direction::MovementDirection,
	system_params::animations::AnimationsContextMut,
};
use bevy::prelude::*;
use common::traits::animation::{MoveDirection, MoveDirectionMut};

impl<TServer> MoveDirection for AnimationsContextMut<'_, TServer> {
	fn move_direction(&self) -> Option<Dir3> {
		self.movement_direction
	}
}

impl<TServer> MoveDirectionMut for AnimationsContextMut<'_, TServer> {
	fn move_direction_mut(&mut self) -> &mut Option<Dir3> {
		&mut self.movement_direction
	}
}

impl<TServer, TAnimationGraph> Drop for AnimationsContextMut<'_, TServer, TAnimationGraph>
where
	TAnimationGraph: Asset,
{
	fn drop(&mut self) {
		let Some(dir) = self.movement_direction else {
			return;
		};

		self.entity.try_insert(MovementDirection(dir));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::system_params::animations::AnimationsParamMut;
	use bevy::{
		animation::graph::AnimationGraph,
		asset::Assets,
		ecs::{
			resource::Resource,
			system::{RunSystemError, RunSystemOnce},
		},
	};
	use common::traits::{accessors::get::GetContextMut, animation::Animations};
	use testing::SingleThreadedApp;

	#[derive(Resource)]
	struct _Server;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(_Server);
		app.insert_resource(Assets::<AnimationGraph>::default());

		app
	}

	#[test]
	fn get_movement_direction() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((GlobalTransform::default(), MovementDirection(Dir3::NEG_Z)))
			.id();

		app.world_mut()
			.run_system_once(move |mut p: AnimationsParamMut<_Server>| {
				let key = Animations { entity };
				let ctx = AnimationsParamMut::get_context_mut(&mut p, key).unwrap();

				assert_eq!(Some(Dir3::NEG_Z), ctx.move_direction());
			})
	}

	#[test]
	fn set_movement_direction() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn(GlobalTransform::default()).id();

		app.world_mut()
			.run_system_once(move |mut p: AnimationsParamMut<_Server>| {
				let key = Animations { entity };
				let mut ctx = AnimationsParamMut::get_context_mut(&mut p, key).unwrap();
				*ctx.move_direction_mut() = Some(Dir3::NEG_Z);
			})?;

		assert_eq!(
			Some(&MovementDirection(Dir3::NEG_Z)),
			app.world().entity(entity).get::<MovementDirection>(),
		);
		Ok(())
	}
}
