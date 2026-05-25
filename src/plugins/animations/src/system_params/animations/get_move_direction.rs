use crate::system_params::animations::AnimationsContextMut;
use bevy::prelude::*;
use common::traits::handles_animations::{GetMoveDirection, GetMoveDirectionMut};

impl GetMoveDirection for AnimationsContextMut<'_> {
	fn get_move_direction(&self) -> Option<Dir3> {
		self.movement_direction.0
	}
}

impl GetMoveDirectionMut for AnimationsContextMut<'_> {
	fn get_move_direction_mut(&mut self) -> &mut Option<Dir3> {
		&mut self.movement_direction.0
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{
		components::{
			animation_dispatch::AnimationDispatch,
			animation_lookup::AnimationLookup,
			current_movement_direction::CurrentMovementDirection,
		},
		system_params::animations::AnimationsParamMut,
	};
	use bevy::{
		animation::graph::AnimationGraph,
		asset::Assets,
		ecs::system::{RunSystemError, RunSystemOnce},
	};
	use common::traits::{
		accessors::get::TryGetContextMut,
		handles_animations::{AnimationClips, Animations},
	};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(Assets::<AnimationGraph>::default());

		app
	}

	#[test]
	fn get_movement_direction() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				AnimationLookup::<AnimationClips<AnimationNodeIndex>>::default(),
				AnimationDispatch::default(),
				GlobalTransform::default(),
				CurrentMovementDirection(Some(Dir3::NEG_Z)),
			))
			.id();

		app.world_mut()
			.run_system_once(move |mut p: AnimationsParamMut| {
				let key = Animations { entity };
				let ctx = AnimationsParamMut::try_get_context_mut(&mut p, key).unwrap();

				assert_eq!(Some(Dir3::NEG_Z), ctx.get_move_direction());
			})
	}

	#[test]
	fn set_movement_direction() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				AnimationLookup::<AnimationClips<AnimationNodeIndex>>::default(),
				AnimationDispatch::default(),
				GlobalTransform::default(),
			))
			.id();

		app.world_mut()
			.run_system_once(move |mut p: AnimationsParamMut| {
				let key = Animations { entity };
				let mut ctx = AnimationsParamMut::try_get_context_mut(&mut p, key).unwrap();
				*ctx.get_move_direction_mut() = Some(Dir3::NEG_Z);
			})?;

		assert_eq!(
			Some(&CurrentMovementDirection(Some(Dir3::NEG_Z))),
			app.world().entity(entity).get::<CurrentMovementDirection>(),
		);
		Ok(())
	}
}
