use crate::{
	components::SlotKey,
	types::{File, Key, SceneId},
};
use bevy::prelude::*;
use std::{borrow::Cow, collections::HashMap, hash::Hash, marker::PhantomData};

#[derive(Resource)]
pub struct Animation<TAgent, TBehavior> {
	agent: PhantomData<TAgent>,
	behavior: PhantomData<TBehavior>,
	pub clip: Handle<AnimationClip>,
}

impl<TAgent, TBehavior> Animation<TAgent, TBehavior> {
	pub fn new(clip: Handle<AnimationClip>) -> Self {
		Self {
			agent: PhantomData,
			behavior: PhantomData,
			clip,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{Player, Run};
	use bevy::{asset::AssetId, utils::Uuid};

	#[test]
	fn set_clip() {
		let clip = Handle::<AnimationClip>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let animation = Animation::<Player, Run>::new(clip.clone_weak());

		assert_eq!(clip, animation.clip);
	}
}

#[derive(Resource)]
pub struct Models(pub HashMap<Cow<'static, str>, Handle<Scene>>);

impl Models {
	pub fn new<const C: usize>(
		pairs: [(&'static Key, &File, SceneId); C],
		asset_server: &Res<AssetServer>,
	) -> Self {
		Models(
			pairs
				.map(|(key, file, scene_id)| {
					(
						Cow::from(key),
						asset_server.load(format!("models/{file}#Scene{scene_id}")),
					)
				})
				.into_iter()
				.collect(),
		)
	}
}

#[derive(Resource)]
pub struct SlotMap<TButton>(pub HashMap<TButton, SlotKey>)
where
	TButton: Eq + Hash;

impl<TButton> SlotMap<TButton>
where
	TButton: Eq + Hash,
{
	pub fn new<const N: usize>(pairs: [(TButton, SlotKey); N]) -> Self {
		Self(pairs.into())
	}
}
