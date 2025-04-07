use bevy::prelude::*;
use common::traits::load_asset::Path;
use std::{collections::HashMap, marker::PhantomData};

#[derive(Resource)]
pub(crate) struct AnimationData<TAgent, TAnimationGraph: Asset = AnimationGraph> {
	_a: PhantomData<TAgent>,
	pub(crate) graph: Handle<TAnimationGraph>,
	pub(crate) animations: HashMap<Path, (AnimationNodeIndex, AnimationMask)>,
}

impl<TAgent, TAnimationGraph: Asset> AnimationData<TAgent, TAnimationGraph> {
	pub(crate) fn new(
		graph: Handle<TAnimationGraph>,
		animations: HashMap<Path, (AnimationNodeIndex, AnimationMask)>,
	) -> Self {
		Self {
			_a: PhantomData,
			graph,
			animations,
		}
	}
}
