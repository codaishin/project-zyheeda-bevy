use crate::{
	components::SlotKey,
	types::{File, Key, SceneId},
};
use bevy::prelude::*;
use std::{collections::HashMap, hash::Hash, marker::PhantomData};

#[derive(Resource)]
pub struct Animation<TAgent, TMarker> {
	phantom_agent: PhantomData<TAgent>,
	phantom_marker: PhantomData<TMarker>,
	pub clip: Handle<AnimationClip>,
}

impl<TAgent, TMarker> Animation<TAgent, TMarker> {
	pub fn new(clip: Handle<AnimationClip>) -> Self {
		Self {
			phantom_agent: PhantomData,
			phantom_marker: PhantomData,
			clip,
		}
	}
}

#[derive(Resource)]
pub struct Models(pub HashMap<&'static Key, Handle<Scene>>);

impl Models {
	pub fn new<const C: usize>(
		pairs: [(&'static Key, &File, SceneId); C],
		asset_server: &Res<AssetServer>,
	) -> Self {
		Models(
			pairs
				.map(|(key, file, scene_id)| {
					(
						key,
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
