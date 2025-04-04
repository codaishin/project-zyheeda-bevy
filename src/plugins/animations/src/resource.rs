use bevy::{
	animation::graph::AnimationMask,
	asset::{Asset, Handle},
	prelude::{AnimationGraph, Resource},
};
use common::traits::load_asset::Path;
use std::{collections::HashMap, marker::PhantomData};

#[derive(Resource)]
pub(crate) struct AnimationData<TAgent, TAnimationGraph: Asset = AnimationGraph> {
	_a: PhantomData<TAgent>,
	pub(crate) graph: Handle<TAnimationGraph>,
	pub(crate) masks: HashMap<Path, AnimationMask>,
}

impl<TAgent, TAnimationGraph: Asset> AnimationData<TAgent, TAnimationGraph> {
	pub(crate) fn new(graph: Handle<TAnimationGraph>, masks: HashMap<Path, AnimationMask>) -> Self {
		Self {
			_a: PhantomData,
			graph,
			masks,
		}
	}
}
