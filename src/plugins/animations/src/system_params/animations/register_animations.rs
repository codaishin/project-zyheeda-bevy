use crate::{
	components::animation_lookup::{AnimationClips, AnimationLookup2, AnimationLookupData},
	system_params::animations::AnimationsContextMut,
	traits::LoadAnimationAssets,
};
use bevy::prelude::*;
use common::traits::{
	animation::{Animation2, AnimationKey, RegisterAnimations2},
	wrap_handle::WrapHandle,
};
use std::collections::HashMap;

impl<TServer, TGraph> RegisterAnimations2 for AnimationsContextMut<'_, TServer, TGraph>
where
	TGraph: Asset + WrapHandle + Sync + Send + 'static,
	TServer: Resource + LoadAnimationAssets<TGraph, AnimationClips>,
{
	fn register_animations(&mut self, animations: HashMap<AnimationKey, Animation2>) {
		let animation_paths = animations
			.values()
			.map(|Animation2 { path, .. }| path.clone())
			.collect::<Vec<_>>();
		let (graph, new_clips) = self.asset_server.load_animation_assets(animation_paths);
		let graph = self.graphs.add(graph);
		let animations = animations
			.into_iter()
			.filter_map(move |(key, animation)| {
				let animation_clips = new_clips.get(&animation.path)?;
				let data = AnimationLookupData {
					animation_clips: *animation_clips,
					play_mode: animation.play_mode,
					mask: animation.mask,
					bones: animation.bones,
				};

				Some((key, data))
			})
			.collect();

		self.entity
			.try_insert((AnimationLookup2 { animations }, TGraph::wrap(graph)));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::system_params::animations::AnimationsParamMut;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		tools::action_key::slot::SlotKey,
		traits::{
			accessors::get::GetContextMut,
			animation::{
				AffectedAnimationBones2,
				AnimationPath,
				Animations as AnimationsKey,
				BoneName,
				PlayMode,
			},
			wrap_handle::UnwrapHandle,
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

		fn wrap(_: Handle<Self>) -> Self::TComponent {
			_GraphComponent
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _GraphComponent;

	impl UnwrapHandle for _GraphComponent {
		type TAsset = _Graph;

		fn unwrap(&self) -> &Handle<Self::TAsset> {
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
				ctx.register_animations(HashMap::default());
			})?;

		assert!(app.world().entity(entity).contains::<_GraphComponent>());
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
				let a = Animation2 {
					path: AnimationPath::from("path/a"),
					play_mode: PlayMode::Repeat,
					mask: 1,
					bones: AffectedAnimationBones2 {
						from_root: BoneName::from("root a"),
						until_exclusive: vec![],
					},
				};
				let b = Animation2 {
					path: AnimationPath::from("path/b"),
					play_mode: PlayMode::Repeat,
					mask: 2,
					bones: AffectedAnimationBones2 {
						from_root: BoneName::from("root b"),
						until_exclusive: vec![],
					},
				};
				let c = Animation2 {
					path: AnimationPath::from("path/c"),
					play_mode: PlayMode::Replay,
					mask: 4,
					bones: AffectedAnimationBones2 {
						from_root: BoneName::from("root c"),
						until_exclusive: vec![],
					},
				};

				ctx.register_animations(HashMap::from([
					(AnimationKey::Idle, a),
					(AnimationKey::Walk, b),
					(AnimationKey::Skill(SlotKey(42)), c),
				]));
			})?;

		assert_eq!(
			Some(&AnimationLookup2 {
				animations: HashMap::from([
					(
						AnimationKey::Idle,
						AnimationLookupData {
							animation_clips: AnimationClips::Single(AnimationNodeIndex::new(1)),
							play_mode: PlayMode::Repeat,
							mask: 1,
							bones: AffectedAnimationBones2 {
								from_root: BoneName::from("root a"),
								until_exclusive: vec![],
							}
						},
					),
					(
						AnimationKey::Walk,
						AnimationLookupData {
							animation_clips: AnimationClips::Single(AnimationNodeIndex::new(2)),
							play_mode: PlayMode::Repeat,
							mask: 2,
							bones: AffectedAnimationBones2 {
								from_root: BoneName::from("root b"),
								until_exclusive: vec![],
							}
						},
					),
					(
						AnimationKey::Skill(SlotKey(42)),
						AnimationLookupData {
							animation_clips: AnimationClips::Single(AnimationNodeIndex::new(3)),
							play_mode: PlayMode::Replay,
							mask: 4,
							bones: AffectedAnimationBones2 {
								from_root: BoneName::from("root c"),
								until_exclusive: vec![],
							}
						},
					),
				])
			}),
			app.world().entity(entity).get::<AnimationLookup2>()
		);
		Ok(())
	}
}
