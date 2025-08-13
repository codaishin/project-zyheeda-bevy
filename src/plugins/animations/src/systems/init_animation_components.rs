use crate::{
	components::animation_lookup::{AnimationLookup, Animations},
	traits::LoadAnimationAssets,
};
use bevy::prelude::*;
use common::{
	traits::{
		accessors::get::TryApplyOn,
		animation::{AnimationMaskDefinition, GetAnimationDefinitions},
		thread_safe::ThreadSafe,
		wrap_handle::WrapHandle,
	},
	zyheeda_commands::ZyheedaCommands,
};
use std::collections::HashMap;

impl<TAgent> InitAnimationComponents for TAgent
where
	TAgent: Component + GetAnimationDefinitions + Sized + ThreadSafe,
	for<'a> AnimationMask: From<&'a Self::TAnimationMask>,
	for<'a> AnimationMaskDefinition: From<&'a Self::TAnimationMask>,
{
}

pub(crate) trait InitAnimationComponents:
	Component + GetAnimationDefinitions + Sized + ThreadSafe
where
	for<'a> AnimationMask: From<&'a Self::TAnimationMask>,
	for<'a> AnimationMaskDefinition: From<&'a Self::TAnimationMask>,
{
	fn init_animation_components<TGraph, TServer>(
		mut commands: ZyheedaCommands,
		mut server: ResMut<TServer>,
		mut graphs: ResMut<Assets<TGraph>>,
		agents: Query<Entity, Added<Self>>,
	) where
		TGraph: Asset + WrapHandle + Sync + Send + 'static,
		TServer: Resource + LoadAnimationAssets<TGraph, Animations>,
	{
		for entity in &agents {
			let animations = Self::animations();
			let assets = animations.keys().cloned().collect::<Vec<_>>();
			let (graph, new_clips) = server.load_animation_assets(assets);
			let graph = graphs.add(graph);
			let lookup = AnimationLookup {
				animations: HashMap::from_iter(new_clips.into_iter().filter_map(
					|(asset, clip)| {
						let mask = animations.get(&asset)?;
						Some((asset, (clip, *mask)))
					},
				)),
			};

			commands.try_apply_on(&entity, |mut e| {
				e.try_insert((lookup, TGraph::wrap(graph)));
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::{
		animation::{AnimationAsset, AnimationMaskDefinition},
		iteration::{Iter, IterFinite},
		thread_safe::ThreadSafe,
		wrap_handle::UnwrapHandle,
	};
	use macros::NestedMocks;
	use mockall::automock;
	use std::collections::{HashMap, HashSet};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Resource, NestedMocks)]
	struct _Server {
		mock: Mock_Server,
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

	#[derive(Clone, Copy)]
	struct _Mask;

	impl IterFinite for _Mask {
		fn iterator() -> Iter<Self> {
			panic!("SHOULD NOT BE USED HERE")
		}

		fn next(_: &Iter<Self>) -> Option<Self> {
			panic!("SHOULD NOT BE USED HERE")
		}
	}

	impl From<&_Mask> for AnimationMask {
		fn from(_: &_Mask) -> Self {
			panic!("SHOULD NOT BE USED HERE")
		}
	}

	impl From<&_Mask> for AnimationMaskDefinition {
		fn from(_: &_Mask) -> Self {
			panic!("SHOULD NOT BE USED HERE")
		}
	}

	#[automock]
	impl LoadAnimationAssets<_Graph, Animations> for _Server {
		fn load_animation_assets(
			&mut self,
			animations: Vec<AnimationAsset>,
		) -> (_Graph, HashMap<AnimationAsset, Animations>) {
			self.mock.load_animation_assets(animations)
		}
	}

	fn setup<TAgent>(server: _Server) -> App
	where
		TAgent: Component + GetAnimationDefinitions + ThreadSafe,
		for<'a> AnimationMask: From<&'a TAgent::TAnimationMask>,
		for<'a> AnimationMaskDefinition: From<&'a TAgent::TAnimationMask>,
	{
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(server);
		app.insert_resource(Assets::<_Graph>::default());
		app.add_systems(Update, TAgent::init_animation_components::<_Graph, _Server>);

		app
	}

	#[test]
	fn add_animation_graph() {
		#[derive(Component, Debug, PartialEq)]
		struct _Agent;

		impl GetAnimationDefinitions for _Agent {
			type TAnimationMask = _Mask;

			fn animations() -> HashMap<AnimationAsset, AnimationMask> {
				HashMap::default()
			}
		}

		let mut app = setup::<_Agent>(_Server::new().with_mock(|mock| {
			mock.expect_load_animation_assets()
				.return_const((_Graph, HashMap::default()));
		}));
		let agent = app.world_mut().spawn(_Agent).id();

		app.update();

		assert!(app.world().entity(agent).contains::<_GraphComponent>());
	}

	#[test]
	fn add_animation_lookup() {
		#[derive(Component, Debug, PartialEq)]
		struct _Agent;

		impl GetAnimationDefinitions for _Agent {
			type TAnimationMask = _Mask;

			fn animations() -> HashMap<AnimationAsset, AnimationMask> {
				HashMap::from([
					(AnimationAsset::from("path/a"), 1),
					(AnimationAsset::from("path/b"), 2),
					(AnimationAsset::from("path/c"), 4),
				])
			}
		}

		let mut app = setup::<_Agent>(_Server::new().with_mock(|mock| {
			mock.expect_load_animation_assets()
				.withf(|paths| {
					assert_eq!(
						HashSet::from([
							&AnimationAsset::from("path/a"),
							&AnimationAsset::from("path/b"),
							&AnimationAsset::from("path/c"),
						]),
						HashSet::from_iter(paths)
					);
					true
				})
				.return_const((
					_Graph,
					HashMap::from([
						(
							AnimationAsset::from("path/a"),
							Animations::Single(AnimationNodeIndex::new(1)),
						),
						(
							AnimationAsset::from("path/b"),
							Animations::Single(AnimationNodeIndex::new(2)),
						),
						(
							AnimationAsset::from("path/c"),
							Animations::Single(AnimationNodeIndex::new(3)),
						),
					]),
				));
		}));
		let agent = app.world_mut().spawn(_Agent).id();

		app.update();

		assert_eq!(
			Some(&AnimationLookup {
				animations: HashMap::from([
					(
						AnimationAsset::from("path/a"),
						(Animations::Single(AnimationNodeIndex::new(1)), 1)
					),
					(
						AnimationAsset::from("path/b"),
						(Animations::Single(AnimationNodeIndex::new(2)), 2)
					),
					(
						AnimationAsset::from("path/c"),
						(Animations::Single(AnimationNodeIndex::new(3)), 4)
					),
				])
			}),
			app.world().entity(agent).get::<AnimationLookup>()
		);
	}

	#[test]
	fn act_only_once() {
		#[derive(Component, Debug, PartialEq)]
		struct _Agent;

		impl GetAnimationDefinitions for _Agent {
			type TAnimationMask = _Mask;

			fn animations() -> HashMap<AnimationAsset, AnimationMask> {
				HashMap::default()
			}
		}

		let mut app = setup::<_Agent>(_Server::new().with_mock(|mock| {
			mock.expect_load_animation_assets()
				.return_const((_Graph, HashMap::default()));
		}));
		let agent = app.world_mut().spawn(_Agent).id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.remove::<(_GraphComponent, AnimationTransitions, AnimationLookup)>();
		app.update();

		assert_eq!(
			[false, false],
			[
				app.world().entity(agent).contains::<_GraphComponent>(),
				app.world().entity(agent).contains::<AnimationLookup>(),
			]
		);
	}
}
