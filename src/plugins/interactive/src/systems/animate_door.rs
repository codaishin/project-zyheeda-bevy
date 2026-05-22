use crate::components::door::Door;
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::traits::{
	accessors::get::{GetContext, GetContextMut},
	handles_animations::{ActiveAnimationsMut, AnimationKey, AnimationPriority, Animations},
	handles_physics::{IsInteracting, IterInteractions},
};
use zyheeda_core::collections::ordered::OrderedSet;

impl Door {
	pub(crate) fn animate<TInteractions, TAnimations>(
		doors: Query<Entity, With<Door>>,
		interactions: StaticSystemParam<TInteractions>,
		mut animations: StaticSystemParam<TAnimations>,
	) where
		TInteractions: for<'c> GetContext<IsInteracting, TContext<'c>: IterInteractions>,
		TAnimations: for<'c> GetContextMut<Animations, TContext<'c>: ActiveAnimationsMut>,
	{
		for entity in doors {
			let key = IsInteracting { entity };
			let interactions = TInteractions::get_context(&interactions, key);

			let key = Animations { entity };
			let Some(mut animations) = TAnimations::get_context_mut(&mut animations, key) else {
				continue;
			};

			let animation = match interactions {
				Some(interactions) if !empty(&interactions) => AnimationKey::Open,
				_ => AnimationKey::Close,
			};

			*animations.active_animations_mut(OpenClose) = OrderedSet::from([animation]);
		}
	}
}

fn empty(interactions: &impl IterInteractions) -> bool {
	interactions.iter_interactions().next().is_none()
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
	use common::traits::{
		handles_animations::{ActiveAnimations, AnimationKey, AnimationPriority},
		handles_map_generation::DoorType,
	};
	use std::{collections::HashMap, iter::Copied};
	use testing::{IsChanged, SingleThreadedApp, fake_entity};
	use zyheeda_core::collections::ordered::OrderedSet;

	#[derive(Component, Default)]
	struct _Interaction(Vec<Entity>);

	impl IterInteractions for _Interaction {
		type TIter<'a>
			= Copied<std::slice::Iter<'a, Entity>>
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

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			(
				Door::animate::<Query<Ref<_Interaction>>, Query<&mut _Animations>>,
				IsChanged::<_Animations>::detect,
			)
				.chain(),
		);

		app
	}

	#[test]
	fn play_only_open_when_interacting() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Door(DoorType::SlideDoor),
				_Interaction(vec![fake_entity!(42)]),
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
	fn play_only_close_when_not_interacting() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Door(DoorType::SlideDoor),
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
	fn play_only_close_when_interactions_empty() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Door(DoorType::SlideDoor),
				_Interaction::default(),
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
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((_Interaction(vec![fake_entity!(42)]), _Animations::default()))
			.id();

		app.update();

		assert_eq!(
			Some(&_Animations::default()),
			app.world().entity(entity).get::<_Animations>(),
		);
	}
}
