use crate::AnimationData;
use bevy::prelude::*;
use common::errors::{Error, Level};
use std::{any::type_name, marker::PhantomData};

impl<T> MaskAnimationNodes for T where T: Component {}

pub(crate) trait MaskAnimationNodes: Component + Sized {
	fn mask_animation_nodes(
		mut graphs: ResMut<Assets<AnimationGraph>>,
		animation_data: Res<AnimationData<Self>>,
	) -> Result<(), NoGraphForAgent<Self>> {
		let Some(graph) = graphs.get_mut(&animation_data.graph) else {
			return Err(NoGraphForAgent(PhantomData));
		};

		for (index, _) in animation_data.animations.values() {
			let Some(animation) = graph.get_mut(*index) else {
				continue;
			};

			animation.add_mask(AnimationMask::MAX);
		}

		Ok(())
	}
}

#[derive(Debug, PartialEq)]
pub(crate) struct NoGraphForAgent<TAgent>(PhantomData<TAgent>);

impl<TAgent> From<NoGraphForAgent<TAgent>> for Error {
	fn from(_: NoGraphForAgent<TAgent>) -> Self {
		Error {
			msg: format!(
				"{}: Does not have any `AnimationData`",
				type_name::<TAgent>()
			),
			lvl: Level::Error,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::AnimationData;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		test_tools::utils::{SingleThreadedApp, new_handle},
		traits::load_asset::Path,
	};
	use std::collections::HashMap;

	fn setup<TAgent>(animations: &[Path]) -> App
	where
		TAgent: Component,
	{
		let mut app = App::new().single_threaded(Update);
		let mut graphs = Assets::<AnimationGraph>::default();
		let mut graph = AnimationGraph::new();
		let clips = HashMap::from_iter(animations.iter().map(|path| {
			(
				path.clone(),
				(
					graph.add_clip(new_handle(), 1., graph.root),
					AnimationMask::default(),
				),
			)
		}));

		app.insert_resource(AnimationData::<TAgent>::new(graphs.add(graph), clips));
		app.insert_resource(graphs);

		app
	}

	#[test]
	fn set_all_animations_to_being_fully_masked() -> Result<(), RunSystemError> {
		#[derive(Component, Debug, PartialEq)]
		struct _Agent;

		let paths = [Path::from("a"), Path::from("b")];
		let mut app = setup::<_Agent>(&paths);

		let result = app
			.world_mut()
			.run_system_once(_Agent::mask_animation_nodes)?;

		let data = app.world().resource::<AnimationData<_Agent>>();
		let graph = app
			.world()
			.resource::<Assets<AnimationGraph>>()
			.get(&data.graph)
			.unwrap();
		let masks = paths.map(|path| {
			let (index, _) = data.animations.get(&path).unwrap();
			graph.get(*index).unwrap().mask
		});
		assert_eq!(
			(Ok(()), [AnimationMask::MAX, AnimationMask::MAX]),
			(result, masks)
		);
		Ok(())
	}

	#[test]
	fn do_not_set_animations_masks_of_other_agent() -> Result<(), RunSystemError> {
		#[derive(Component)]
		struct _Agent;

		#[derive(Component)]
		struct _OtherAgent;

		let paths = [Path::from("a"), Path::from("b")];
		let mut app = setup::<_OtherAgent>(&paths);
		app.insert_resource(AnimationData::<_Agent>::new(
			new_handle(),
			HashMap::default(),
		));

		_ = app
			.world_mut()
			.run_system_once(_Agent::mask_animation_nodes)?;

		let data = app.world().resource::<AnimationData<_OtherAgent>>();
		let graph = app
			.world()
			.resource::<Assets<AnimationGraph>>()
			.get(&data.graph)
			.unwrap();
		let masks = paths.map(|path| {
			let (index, _) = data.animations.get(&path).unwrap();
			graph.get(*index).unwrap().mask
		});
		assert_eq!([AnimationMask::default(), AnimationMask::default()], masks);
		Ok(())
	}

	#[test]
	fn return_no_graph_error() -> Result<(), RunSystemError> {
		#[derive(Component, Debug, PartialEq)]
		struct _Agent;

		let mut app = setup::<_Agent>(&[]);
		app.world_mut()
			.resource_mut::<AnimationData<_Agent>>()
			.graph = new_handle();

		let error = app
			.world_mut()
			.run_system_once(_Agent::mask_animation_nodes)?;

		assert_eq!(Err(NoGraphForAgent(PhantomData::<_Agent>)), error);
		Ok(())
	}
}
