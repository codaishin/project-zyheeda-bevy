use crate::system_params::animations::AnimationsContextMut;
use common::traits::handles_animations::{
	ActiveAnimations,
	ActiveAnimationsMut,
	AnimationKey,
	AnimationPriority,
};
use zyheeda_core::prelude::*;

impl ActiveAnimations for AnimationsContextMut<'_> {
	fn active_animations<TLayer>(&self, layer: TLayer) -> &OrderedSet<AnimationKey>
	where
		TLayer: Into<AnimationPriority>,
	{
		self.dispatch.slot(layer)
	}
}

impl ActiveAnimationsMut for AnimationsContextMut<'_> {
	fn active_animations_mut<TLayer>(&mut self, layer: TLayer) -> &mut OrderedSet<AnimationKey>
	where
		TLayer: Into<AnimationPriority>,
	{
		self.dispatch.slot_mut(layer)
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{
		components::{animation_dispatch::AnimationDispatch, animation_lookup::AnimationLookup},
		system_params::animations::AnimationsParamMut,
	};
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::{
		tools::action_key::slot::SlotKey,
		traits::{
			accessors::get::TryGetContextMut,
			handles_animations::{AnimationClips, Animations, SkillAnimation},
		},
	};
	use test_case::test_case;
	use testing::SingleThreadedApp;

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

		app.insert_resource(Assets::<AnimationGraph>::default());

		app
	}

	#[test_case(AnimationPriority::Low; "low")]
	#[test_case(AnimationPriority::Medium; "medium")]
	#[test_case(AnimationPriority::High; "high")]
	fn get_mut_animations(animation_priority: AnimationPriority) -> Result<(), RunSystemError> {
		let mut app = setup();
		let animations = [
			AnimationKey::Run,
			AnimationKey::Skill {
				slot: SlotKey(11),
				animation: SkillAnimation::Aim,
			},
		];
		let entity = app
			.world_mut()
			.spawn(AnimationLookup::<AnimationClips<AnimationNodeIndex>>::default())
			.id();

		app.world_mut()
			.run_system_once(move |mut p: AnimationsParamMut| {
				let key = Animations { entity };
				let mut ctx = AnimationsParamMut::try_get_context_mut(&mut p, key).unwrap();
				ctx.active_animations_mut(animation_priority)
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
		let animations = [
			AnimationKey::Run,
			AnimationKey::Skill {
				slot: SlotKey(11),
				animation: SkillAnimation::Aim,
			},
		];
		let entity = app
			.world_mut()
			.spawn((
				AnimationLookup::<AnimationClips<AnimationNodeIndex>>::default(),
				dispatch_with([(animation_priority, animations)]),
			))
			.id();

		app.world_mut()
			.run_system_once(move |mut p: AnimationsParamMut| {
				let key = Animations { entity };
				let ctx = AnimationsParamMut::try_get_context_mut(&mut p, key).unwrap();

				assert_eq!(
					&OrderedSet::from(animations),
					ctx.active_animations(animation_priority)
				);
			})
	}
}
