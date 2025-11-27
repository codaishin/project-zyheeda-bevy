use crate::system_params::animations::AnimationsContextMut;
use common::traits::handles_animations::{
	ActiveAnimations,
	ActiveAnimationsMut,
	AnimationKey,
	AnimationPriority,
	AnimationsUnprepared,
};
use std::collections::HashSet;

impl<TServer> ActiveAnimations for AnimationsContextMut<'_, TServer> {
	fn active_animations<TLayer>(
		&self,
		layer: TLayer,
	) -> Result<&HashSet<AnimationKey>, AnimationsUnprepared>
	where
		TLayer: Into<AnimationPriority>,
	{
		match &self.dispatch {
			Some(dispatch) => Ok(dispatch.slot(layer)),
			None => Err(AnimationsUnprepared {
				entity: self.entity.id(),
			}),
		}
	}
}

impl<TServer> ActiveAnimationsMut for AnimationsContextMut<'_, TServer> {
	fn active_animations_mut<TLayer>(
		&mut self,
		layer: TLayer,
	) -> Result<&mut HashSet<AnimationKey>, AnimationsUnprepared>
	where
		TLayer: Into<AnimationPriority>,
	{
		match &mut self.dispatch {
			Some(dispatch) => Ok(dispatch.slot_mut(layer)),
			None => Err(AnimationsUnprepared {
				entity: self.entity.id(),
			}),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::animation_dispatch::AnimationDispatch,
		system_params::animations::AnimationsParamMut,
	};
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::{
		tools::action_key::slot::SlotKey,
		traits::{accessors::get::GetContextMut, handles_animations::Animations},
	};
	use test_case::test_case;
	use testing::SingleThreadedApp;

	#[derive(Resource)]
	struct _Server;

	fn dispatch_with<TAnimations, TLayer, TKeys>(animations: TAnimations) -> AnimationDispatch
	where
		TAnimations: IntoIterator<Item = (TLayer, TKeys)>,
		TLayer: Into<AnimationPriority> + Copy,
		TKeys: IntoIterator<Item = AnimationKey>,
	{
		let mut dispatch = AnimationDispatch::default();
		for (layer, keys) in animations {
			dispatch.slot_mut(layer).extend(keys);
		}
		dispatch
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(_Server);
		app.insert_resource(Assets::<AnimationGraph>::default());

		app
	}

	#[test_case(AnimationPriority::Low; "low")]
	#[test_case(AnimationPriority::Medium; "medium")]
	#[test_case(AnimationPriority::High; "high")]
	fn get_mut_animations(animation_priority: AnimationPriority) -> Result<(), RunSystemError> {
		let mut app = setup();
		let animations = [AnimationKey::Run, AnimationKey::Skill(SlotKey(11))];
		let entity = app.world_mut().spawn(AnimationDispatch::default()).id();

		app.world_mut()
			.run_system_once(move |mut p: AnimationsParamMut<_Server>| {
				let key = Animations { entity };
				let mut ctx = AnimationsParamMut::get_context_mut(&mut p, key).unwrap();
				ctx.active_animations_mut(animation_priority)
					.unwrap()
					.extend(animations);
			})?;

		assert_eq!(
			Some(&dispatch_with([(animation_priority, animations)])),
			app.world().entity(entity).get::<AnimationDispatch>(),
		);
		Ok(())
	}

	#[test_case(AnimationPriority::Low; "low")]
	#[test_case(AnimationPriority::Medium; "medium")]
	#[test_case(AnimationPriority::High; "high")]
	fn get_animations(animation_priority: AnimationPriority) -> Result<(), RunSystemError> {
		let mut app = setup();
		let animations = [AnimationKey::Run, AnimationKey::Skill(SlotKey(11))];
		let entity = app
			.world_mut()
			.spawn(dispatch_with([(animation_priority, animations)]))
			.id();

		app.world_mut()
			.run_system_once(move |mut p: AnimationsParamMut<_Server>| {
				let key = Animations { entity };
				let ctx = AnimationsParamMut::get_context_mut(&mut p, key).unwrap();

				assert_eq!(
					Ok(&HashSet::from(animations)),
					ctx.active_animations(animation_priority)
				);
			})
	}
}
