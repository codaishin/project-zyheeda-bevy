use std::{borrow::Cow, marker::PhantomData};

use bevy::{prelude::*, utils::HashMap};

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
	use bevy::{asset::HandleId, utils::Uuid};

	#[test]
	fn set_clip() {
		let clip = Handle::<AnimationClip>::weak(HandleId::new(Uuid::new_v4(), 42));
		let animation = Animation::<Player, Run>::new(clip.clone_weak());

		assert_eq!(clip, animation.clip);
	}
}

#[derive(Resource)]
pub struct Models(pub HashMap<Cow<'static, str>, Handle<Scene>>);

type Key = str;
type File = str;
type SceneId = u8;

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
