use crate::{
	components::animation_lookup::AnimationLookup2,
	traits::asset_server::animation_graph::GetNodeMut,
};
use bevy::prelude::*;
use common::traits::{iterate::Iterate, thread_safe::ThreadSafe, wrap_handle::UnwrapHandle};

impl<T> MaskAllBits for T where T: Component + UnwrapHandle<TAsset: GetNodeMut> {}

pub(crate) trait MaskAllBits: Component + UnwrapHandle<TAsset: GetNodeMut> + Sized {
	fn on_add_mask_all_bits_for<TAnimations>(
		graphs: Query<(&Self, &AnimationLookup2<TAnimations>), Added<Self>>,
		mut assets: ResMut<Assets<Self::TAsset>>,
	) where
		TAnimations: for<'a> Iterate<'a, TItem = &'a AnimationNodeIndex> + ThreadSafe,
	{
		for (graph, lookup) in &graphs {
			let handle = graph.unwrap();
			let Some(graph) = assets.get_mut(handle) else {
				continue;
			};

			for data in lookup.animations.values() {
				for clip in data.animation_clips.iterate() {
					let Some(node) = graph.get_node_mut(*clip) else {
						continue;
					};

					node.add_mask(AnimationMask::MAX);
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::animation_lookup::{AnimationLookup2, AnimationLookupData};
	use common::traits::animation::AnimationKey;
	use std::{collections::HashMap, slice::Iter};
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Component)]
	struct _Component(Handle<_Asset>);

	impl UnwrapHandle for _Component {
		type TAsset = _Asset;

		fn unwrap(&self) -> &Handle<Self::TAsset> {
			&self.0
		}
	}

	#[derive(Asset, TypePath)]
	struct _Asset(HashMap<AnimationNodeIndex, AnimationGraphNode>);

	impl GetNodeMut for _Asset {
		fn get_node_mut(
			&mut self,
			animation: AnimationNodeIndex,
		) -> Option<&mut AnimationGraphNode> {
			self.0.get_mut(&animation)
		}
	}

	#[derive(Default)]
	struct _Animations(Vec<AnimationNodeIndex>);

	impl<'a> Iterate<'a> for _Animations {
		type TItem = &'a AnimationNodeIndex;
		type TIter = Iter<'a, AnimationNodeIndex>;

		fn iterate(&'a self) -> Self::TIter {
			self.0.iter()
		}
	}

	fn setup<const N: usize>(assets: [(&Handle<_Asset>, _Asset); N]) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut asset_resource = Assets::default();

		for (id, asset) in assets {
			asset_resource.insert(id, asset);
		}

		app.insert_resource(asset_resource);
		app.add_systems(Update, _Component::on_add_mask_all_bits_for::<_Animations>);

		app
	}

	#[test]
	fn mask_all_nodes() {
		let handle = new_handle();
		let asset = _Asset(HashMap::from([(
			AnimationNodeIndex::new(42),
			AnimationGraphNode::default(),
		)]));
		let mut app = setup([(&handle, asset)]);
		app.world_mut().spawn((
			AnimationLookup2 {
				animations: HashMap::from([(
					AnimationKey::Run,
					AnimationLookupData {
						animation_clips: _Animations(vec![AnimationNodeIndex::new(42)]),
						..default()
					},
				)]),
			},
			_Component(handle.clone()),
		));

		app.update();

		assert_eq!(
			Some(AnimationMask::MAX),
			app.world()
				.resource::<Assets<_Asset>>()
				.get(&handle)
				.and_then(|n| n.0.get(&AnimationNodeIndex::new(42)))
				.map(|n| n.mask)
		);
	}

	#[test]
	fn act_only_once() {
		let handle = new_handle();
		let asset = _Asset(HashMap::from([(
			AnimationNodeIndex::new(42),
			AnimationGraphNode::default(),
		)]));
		let mut app = setup([(&handle, asset)]);
		app.world_mut().spawn((
			AnimationLookup2 {
				animations: HashMap::from([(
					AnimationKey::Run,
					AnimationLookupData {
						animation_clips: _Animations(vec![AnimationNodeIndex::new(42)]),
						..default()
					},
				)]),
			},
			_Component(handle.clone()),
		));

		app.update();
		let mut graphs = app.world_mut().resource_mut::<Assets<_Asset>>();
		let graph = graphs.get_mut(&handle).unwrap();
		let node = graph.0.get_mut(&AnimationNodeIndex::new(42)).unwrap();
		node.mask = 0;
		app.update();

		assert_eq!(
			Some(0),
			app.world()
				.resource::<Assets<_Asset>>()
				.get(&handle)
				.and_then(|n| n.0.get(&AnimationNodeIndex::new(42)))
				.map(|n| n.mask)
		);
	}
}
