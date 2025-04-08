use bevy::prelude::*;
use common::traits::animation::AnimationAsset;
use std::{collections::HashMap, marker::PhantomData};

#[derive(Resource)]
pub(crate) struct AnimationData<TAgent, TAnimationGraph = AnimationGraph>
where
	TAnimationGraph: Asset,
{
	_a: PhantomData<TAgent>,
	pub(crate) graph: Handle<TAnimationGraph>,
	pub(crate) animations: HashMap<AnimationAsset, (Vec<AnimationNodeIndex>, AnimationMask)>,
}

impl<TAgent, TAnimationGraph> AnimationData<TAgent, TAnimationGraph>
where
	TAnimationGraph: Asset,
{
	pub(crate) fn new(
		graph: Handle<TAnimationGraph>,
		animations: HashMap<AnimationAsset, (Vec<AnimationNodeIndex>, AnimationMask)>,
	) -> Self {
		Self {
			_a: PhantomData,
			graph,
			animations,
		}
	}
}
