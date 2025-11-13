use crate::{
	components::movement_direction::MovementDirection,
	system_params::animations::AnimationsContextMut,
};
use bevy::prelude::*;
use common::traits::animation::SetMovementDirection;

impl<TServer> SetMovementDirection for AnimationsContextMut<'_, TServer> {
	fn set_movement_direction(&mut self, direction: Dir3) {
		self.entity.try_insert(MovementDirection(direction));
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
	fn insert_movement_forward_from_direction() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn(GlobalTransform::default()).id();

		app.world_mut()
			.run_system_once(move |mut p: AnimationsParamMut<_Server>| {
				let key = Animations { entity };
				let mut ctx = AnimationsParamMut::get_context_mut(&mut p, key).unwrap();
				ctx.set_movement_direction(Dir3::NEG_Z);
			})?;

		assert_eq!(
			Some(&MovementDirection(Dir3::NEG_Z)),
			app.world().entity(entity).get::<MovementDirection>(),
		);
		Ok(())
	}
}
