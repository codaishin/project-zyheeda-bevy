use bevy::{input::keyboard::KeyCode, prelude::States};
use std::{
	fmt::Debug,
	hash::{Hash, Hasher},
	marker::PhantomData,
	ops::Deref,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, States)]
pub enum MouseContext<TKey = KeyCode>
where
	TKey: Debug + Hash + Eq + Clone + Sync + Send + 'static,
{
	#[default]
	Default,
	UI,
	Primed(TKey),
	JustTriggered(TKey),
	Triggered(TKey),
	JustReleased(TKey),
}

#[derive(States, Debug)]
pub struct AssetLoadState<TAsset: Debug + Send + Sync + 'static> {
	phantom_data: PhantomData<TAsset>,
	state: LoadState,
}

impl<TAsset: Debug + Send + Sync + 'static> Deref for AssetLoadState<TAsset> {
	type Target = LoadState;

	fn deref(&self) -> &Self::Target {
		&self.state
	}
}

impl<TAsset: Debug + Send + Sync + 'static> Clone for AssetLoadState<TAsset> {
	fn clone(&self) -> Self {
		Self {
			phantom_data: self.phantom_data,
			state: self.state.clone(),
		}
	}
}

impl<TAsset: Debug + Send + Sync + 'static> PartialEq for AssetLoadState<TAsset> {
	fn eq(&self, other: &Self) -> bool {
		self.state == other.state
	}
}

impl<TAsset: Debug + Send + Sync + 'static> Eq for AssetLoadState<TAsset> {}

impl<TAsset: Debug + Send + Sync + 'static> Hash for AssetLoadState<TAsset> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.state.hash(state);
	}
}

impl<TAsset: Debug + Send + Sync + 'static> AssetLoadState<TAsset> {
	pub fn new(value: LoadState) -> Self {
		Self {
			phantom_data: PhantomData,
			state: value,
		}
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum LoadState {
	Loading,
	Loaded,
}
