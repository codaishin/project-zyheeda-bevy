use crate::{
	components::{
		animation_dispatch::AnimationDispatch,
		animation_lookup::{AnimationClips, AnimationLookup, AnimationLookupData},
	},
	system_params::animations::AnimationsContextMut,
	traits::LoadAnimationAssets,
};
use bevy::prelude::*;
use common::traits::{
	handles_animations::{
		AffectedAnimationBones,
		Animation,
		AnimationKey,
		AnimationMaskBits,
		RegisterAnimations,
	},
	wrap_handle::WrapHandle,
};
use std::collections::HashMap;

impl<TServer, TGraph> RegisterAnimations for AnimationsContextMut<'_, TServer, TGraph>
where
	TGraph: Asset + WrapHandle + Sync + Send + 'static,
	TServer: Resource + LoadAnimationAssets<TGraph, AnimationClips>,
{
	fn register_animations(
		&mut self,
		animations: &HashMap<AnimationKey, Animation>,
		animation_mask_groups: &HashMap<AnimationMaskBits, AffectedAnimationBones>,
	) {
		let animation_paths = animations
			.values()
			.map(|Animation { path, .. }| path.clone())
			.collect::<Vec<_>>();
		let animation_mask_groups = animation_mask_groups.clone();
		let (graph, new_clips) = self.asset_server.load_animation_assets(animation_paths);
		let animations = animations
			.iter()
			.filter_map(move |(key, animation)| {
				let animation_clips = new_clips.get(&animation.path)?;
				let data = AnimationLookupData {
					animation_clips: *animation_clips,
					play_mode: animation.play_mode,
					mask: animation.mask_groups,
				};

				Some((*key, data))
			})
			.collect();

		self.entity.try_insert((
			AnimationDispatch::default(),
			AnimationLookup {
				animations,
				animation_mask_groups,
			},
			TGraph::wrap_handle(self.graphs.add(graph)),
		));
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{
		components::animation_dispatch::AnimationDispatch,
		system_params::animations::AnimationsParamMut,
	};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		bit_mask_index,
		tools::{action_key::slot::SlotKey, bone_name::BoneName},
		traits::{
			accessors::get::GetContextMut,
			handles_animations::{
				AffectedAnimationBones,
				AnimationPath,
				Animations as AnimationsKey,
				PlayMode,
			},
			wrap_handle::GetHandle,
		},
	};
	use macros::NestedMocks;
	use mockall::automock;
	use std::collections::HashSet;
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Resource, NestedMocks)]
	struct _Server {
		mock: Mock_Server,
	}

	#[automock]
	impl LoadAnimationAssets<_Graph, AnimationClips> for _Server {
		fn load_animation_assets(
			&mut self,
			animations: Vec<AnimationPath>,
		) -> (_Graph, HashMap<AnimationPath, AnimationClips>) {
			self.mock.load_animation_assets(animations)
		}
	}

	#[derive(Debug, PartialEq, Clone, TypePath, Asset)]
	struct _Graph;

	impl WrapHandle for _Graph {
		type TComponent = _GraphComponent;

		fn wrap_handle(_: Handle<Self>) -> Self::TComponent {
			_GraphComponent
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _GraphComponent;

	impl GetHandle for _GraphComponent {
		type TAsset = _Graph;

		fn get_handle(&self) -> &Handle<Self::TAsset> {
			panic!("NOT USED HERE")
		}
	}

	fn setup(server: _Server) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(server);
		app.insert_resource(Assets::<_Graph>::default());

		app
	}

	#[test]
	fn add_animation_graph() -> Result<(), RunSystemError> {
		let mut app = setup(_Server::new().with_mock(|mock| {
			mock.expect_load_animation_assets()
				.return_const((_Graph, HashMap::default()));
		}));
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut p: AnimationsParamMut<_Server, _Graph>| {
				let key = AnimationsKey { entity };
				let mut ctx = AnimationsParamMut::get_context_mut(&mut p, key).unwrap();
				ctx.register_animations(&HashMap::default(), &HashMap::default());
			})?;

		assert!(app.world().entity(entity).contains::<_GraphComponent>());
		Ok(())
	}

	#[test]
	fn add_animation_dispatch() -> Result<(), RunSystemError> {
		let mut app = setup(_Server::new().with_mock(|mock| {
			mock.expect_load_animation_assets()
				.return_const((_Graph, HashMap::default()));
		}));
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut p: AnimationsParamMut<_Server, _Graph>| {
				let key = AnimationsKey { entity };
				let mut ctx = AnimationsParamMut::get_context_mut(&mut p, key).unwrap();
				ctx.register_animations(&HashMap::default(), &HashMap::default());
			})?;

		assert_eq!(
			Some(&AnimationDispatch::default()),
			app.world().entity(entity).get::<AnimationDispatch>()
		);
		Ok(())
	}

	#[test]
	fn add_animation_lookup() -> Result<(), RunSystemError> {
		let mut app = setup(_Server::new().with_mock(|mock| {
			mock.expect_load_animation_assets()
				.withf(|paths| {
					assert_eq!(
						HashSet::from([
							&AnimationPath::from("path/a"),
							&AnimationPath::from("path/b"),
							&AnimationPath::from("path/c"),
						]),
						HashSet::from_iter(paths)
					);
					true
				})
				.return_const((
					_Graph,
					HashMap::from([
						(
							AnimationPath::from("path/a"),
							AnimationClips::Single(AnimationNodeIndex::new(1)),
						),
						(
							AnimationPath::from("path/b"),
							AnimationClips::Single(AnimationNodeIndex::new(2)),
						),
						(
							AnimationPath::from("path/c"),
							AnimationClips::Single(AnimationNodeIndex::new(3)),
						),
					]),
				));
		}));
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut p: AnimationsParamMut<_Server, _Graph>| {
				let key = AnimationsKey { entity };
				let mut ctx = AnimationsParamMut::get_context_mut(&mut p, key).unwrap();
				let a = Animation {
					path: AnimationPath::from("path/a"),
					play_mode: PlayMode::Repeat,
					mask_groups: AnimationMaskBits::zero().with_set(bit_mask_index!(0)),
				};
				let b = Animation {
					path: AnimationPath::from("path/b"),
					play_mode: PlayMode::Repeat,
					mask_groups: AnimationMaskBits::zero().with_set(bit_mask_index!(1)),
				};
				let c = Animation {
					path: AnimationPath::from("path/c"),
					play_mode: PlayMode::Replay,
					mask_groups: AnimationMaskBits::zero().with_set(bit_mask_index!(2)),
				};

				ctx.register_animations(
					&HashMap::from([
						(AnimationKey::Idle, a),
						(AnimationKey::Walk, b),
						(AnimationKey::Skill(SlotKey(42)), c),
					]),
					&HashMap::from([
						(
							AnimationMaskBits::zero().with_set(bit_mask_index!(0)),
							AffectedAnimationBones {
								from_root: BoneName::from("root a"),
								..default()
							},
						),
						(
							AnimationMaskBits::zero().with_set(bit_mask_index!(1)),
							AffectedAnimationBones {
								from_root: BoneName::from("root b"),
								..default()
							},
						),
						(
							AnimationMaskBits::zero().with_set(bit_mask_index!(2)),
							AffectedAnimationBones {
								from_root: BoneName::from("root c"),
								..default()
							},
						),
					]),
				);
			})?;

		assert_eq!(
			Some(&AnimationLookup {
				animations: HashMap::from([
					(
						AnimationKey::Idle,
						AnimationLookupData {
							animation_clips: AnimationClips::Single(AnimationNodeIndex::new(1)),
							play_mode: PlayMode::Repeat,
							mask: AnimationMaskBits::zero().with_set(bit_mask_index!(0)),
						},
					),
					(
						AnimationKey::Walk,
						AnimationLookupData {
							animation_clips: AnimationClips::Single(AnimationNodeIndex::new(2)),
							play_mode: PlayMode::Repeat,
							mask: AnimationMaskBits::zero().with_set(bit_mask_index!(1)),
						},
					),
					(
						AnimationKey::Skill(SlotKey(42)),
						AnimationLookupData {
							animation_clips: AnimationClips::Single(AnimationNodeIndex::new(3)),
							play_mode: PlayMode::Replay,
							mask: AnimationMaskBits::zero().with_set(bit_mask_index!(2)),
						},
					),
				]),
				animation_mask_groups: HashMap::from([
					(
						AnimationMaskBits::zero().with_set(bit_mask_index!(0)),
						AffectedAnimationBones {
							from_root: BoneName::from("root a"),
							..default()
						},
					),
					(
						AnimationMaskBits::zero().with_set(bit_mask_index!(1)),
						AffectedAnimationBones {
							from_root: BoneName::from("root b"),
							..default()
						},
					),
					(
						AnimationMaskBits::zero().with_set(bit_mask_index!(2)),
						AffectedAnimationBones {
							from_root: BoneName::from("root c"),
							..default()
						},
					),
				]),
			}),
			app.world().entity(entity).get::<AnimationLookup>()
		);
		Ok(())
	}
}
