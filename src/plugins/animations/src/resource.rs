use bevy::{
	asset::{Asset, Handle},
	prelude::{AnimationGraph, Resource},
};
use std::marker::PhantomData;

#[derive(Resource)]
pub(crate) struct AnimationData<TAgent, TAnimationGraph: Asset = AnimationGraph> {
	agent: PhantomData<TAgent>,
	pub(crate) graph: Handle<TAnimationGraph>,
}

impl<TAgent, TAnimationGraph: Asset> AnimationData<TAgent, TAnimationGraph> {
	pub(crate) fn new(handle: Handle<TAnimationGraph>) -> Self {
		Self {
			agent: PhantomData,
			graph: handle,
		}
	}
}
