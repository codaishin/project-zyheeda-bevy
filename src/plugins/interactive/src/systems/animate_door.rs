use crate::components::door::Door;
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::traits::{
	accessors::get::{ContextChanged, GetContext, TryGetContextMut},
	handles_animations::{ActiveAnimationsMut, AnimationKey, AnimationPriority, Animations},
	handles_physics::{Interactions, IterInteractions},
};
use zyheeda_core::collections::ordered::OrderedSet;

impl Door {
	pub(crate) fn animate<TInteractions, TAnimations>(
		doors: Query<Entity, With<Door>>,
		interactions: StaticSystemParam<TInteractions>,
		mut animations: StaticSystemParam<TAnimations>,
	) where
		TInteractions: for<'c> GetContext<Interactions, TContext<'c>: IterInteractions>,
		TAnimations: for<'c> TryGetContextMut<Animations, TContext<'c>: ActiveAnimationsMut>,
	{
		for entity in doors {
			let key = Interactions { entity };
			let interactions = TInteractions::get_context(&interactions, key);
			if !interactions.context_changed() {
				continue;
			}

			let key = Animations { entity };
			let Some(mut animations) = TAnimations::try_get_context_mut(&mut animations, key)
			else {
				continue;
			};

			let animation = match interactions.iter_interactions().len() {
				0 => AnimationKey::Close,
				_ => AnimationKey::Open,
			};

			*animations.active_animations_mut(OpenClose) = OrderedSet::from([animation]);
		}
	}
}

struct OpenClose;

impl From<OpenClose> for AnimationPriority {
	fn from(_: OpenClose) -> Self {
		Self::High
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::handles_animations::{ActiveAnimations, AnimationKey, AnimationPriority};
	use std::{collections::HashMap, iter::Copied, ops::DerefMut, slice::Iter};
	use testing::{IsChanged, SingleThreadedApp, fake_entity};
	use zyheeda_core::collections::ordered::OrderedSet;

	#[derive(Resource, Default)]
	struct _Interaction(Vec<Entity>);

	impl IterInteractions for _Interaction {
		type TIter<'a>
			= Copied<Iter<'a, Entity>>
		where
			Self: 'a;

		fn iter_interactions(&self) -> Self::TIter<'_> {
			self.0.iter().copied()
		}
	}

	#[derive(Component, Debug, PartialEq, Default)]
	struct _Animations(HashMap<AnimationPriority, OrderedSet<AnimationKey>>);

	impl ActiveAnimations for _Animations {
		fn active_animations<TLayer>(&self, layer: TLayer) -> &OrderedSet<AnimationKey>
		where
			TLayer: Into<AnimationPriority>,
		{
			self.0
				.get(&layer.into())
				.unwrap_or(const { &OrderedSet::EMPTY })
		}
	}

	impl ActiveAnimationsMut for _Animations {
		fn active_animations_mut<TLayer>(&mut self, layer: TLayer) -> &mut OrderedSet<AnimationKey>
		where
			TLayer: Into<AnimationPriority>,
		{
			self.0.entry(layer.into()).or_default()
		}
	}

	fn setup(interactions: _Interaction) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(interactions);
		app.add_systems(
			Update,
			(
				Door::animate::<Res<_Interaction>, Query<&mut _Animations>>,
				IsChanged::<_Animations>::detect,
			)
				.chain(),
		);

		app
	}

	#[test]
	fn play_only_open_when_interacting() {
		let mut app = setup(_Interaction(vec![fake_entity!(42)]));
		let entity = app
			.world_mut()
			.spawn((
				Door,
				_Animations(HashMap::from([(
					AnimationPriority::High,
					OrderedSet::from([AnimationKey::Close, AnimationKey::Idle]),
				)])),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Animations(HashMap::from([(
				AnimationPriority::High,
				OrderedSet::from([AnimationKey::Open])
			)]))),
			app.world().entity(entity).get::<_Animations>(),
		);
	}

	#[test]
	fn play_only_close_when_interactions_empty() {
		let mut app = setup(_Interaction::default());
		let entity = app
			.world_mut()
			.spawn((
				Door,
				_Animations(HashMap::from([(
					AnimationPriority::High,
					OrderedSet::from([AnimationKey::Open, AnimationKey::Idle]),
				)])),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Animations(HashMap::from([(
				AnimationPriority::High,
				OrderedSet::from([AnimationKey::Close])
			)]))),
			app.world().entity(entity).get::<_Animations>(),
		);
	}

	#[test]
	fn do_nothing_when_door_missing() {
		let mut app = setup(_Interaction(vec![fake_entity!(42)]));
		let entity = app.world_mut().spawn(_Animations::default()).id();

		app.update();

		assert_eq!(
			Some(&_Animations::default()),
			app.world().entity(entity).get::<_Animations>(),
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup(_Interaction(vec![fake_entity!(42)]));
		let entity = app.world_mut().spawn((Door, _Animations::default())).id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(_Animations::default());
		app.update();

		assert_eq!(
			Some(&_Animations::default()),
			app.world().entity(entity).get::<_Animations>(),
		);
	}

	#[test]
	fn act_again_if_interactions_changed() {
		let mut app = setup(_Interaction(vec![fake_entity!(42)]));
		let entity = app.world_mut().spawn((Door, _Animations::default())).id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(_Animations::default());
		app.world_mut().resource_mut::<_Interaction>().deref_mut();
		app.update();

		assert_eq!(
			Some(&_Animations(HashMap::from([(
				AnimationPriority::High,
				OrderedSet::from([AnimationKey::Open]),
			)]))),
			app.world().entity(entity).get::<_Animations>(),
		);
	}
}
