use crate::system_params::animations::AnimationsContextMut;
use common::traits::handles_animations::{DirForwardPitch, GetForwardPitch, GetForwardPitchMut};

impl GetForwardPitch for AnimationsContextMut<'_> {
	fn get_forward_pitch(&self) -> Option<DirForwardPitch> {
		self.forward_pitch.0
	}
}

impl GetForwardPitchMut for AnimationsContextMut<'_> {
	fn get_forward_pitch_mut(&mut self) -> &mut Option<DirForwardPitch> {
		&mut self.forward_pitch.0
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{
		components::{
			animation_dispatch::AnimationDispatch,
			current_forward_pitch::CurrentForwardPitch,
		},
		system_params::animations::AnimationsParamMut,
	};
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::traits::{
		accessors::get::GetContextMut,
		handles_animations::{Animations, ForwardPitch},
	};
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
	fn get_forward_pitch() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				AnimationDispatch::default(),
				GlobalTransform::default(),
				CurrentForwardPitch(Some(DirForwardPitch::Up(
					ForwardPitch::try_from(0.4).unwrap(),
				))),
			))
			.id();

		app.world_mut()
			.run_system_once(move |mut p: AnimationsParamMut<_Server>| {
				let key = Animations { entity };
				let ctx = AnimationsParamMut::get_context_mut(&mut p, key).unwrap();

				assert_eq!(
					Some(DirForwardPitch::Up(ForwardPitch::try_from(0.4).unwrap())),
					ctx.get_forward_pitch(),
				);
			})
	}

	#[test]
	fn set_forward_pitch() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((AnimationDispatch::default(), GlobalTransform::default()))
			.id();

		app.world_mut()
			.run_system_once(move |mut p: AnimationsParamMut<_Server>| {
				let key = Animations { entity };
				let mut ctx = AnimationsParamMut::get_context_mut(&mut p, key).unwrap();
				*ctx.get_forward_pitch_mut() =
					Some(DirForwardPitch::Up(ForwardPitch::try_from(0.4).unwrap()));
			})?;

		assert_eq!(
			Some(&CurrentForwardPitch(Some(DirForwardPitch::Up(
				ForwardPitch::try_from(0.4).unwrap()
			)))),
			app.world().entity(entity).get::<CurrentForwardPitch>(),
		);
		Ok(())
	}
}
