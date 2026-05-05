use super::InsertClips;
use bevy::prelude::*;
use common::traits::handles_animations::{AnimationClips, Directional, PitchedForward};

impl<TGraph> InsertClips<AnimationClips<AnimationNodeIndex>> for TGraph
where
	TGraph: AnimationGraphTrait + Default,
{
	type TBuffer = AnimationNodeIndex;

	fn with_buffer() -> (Self, Self::TBuffer) {
		let mut graph = Self::default();
		let blend_node = graph.add_additive_blend(1., graph.root());

		(graph, blend_node)
	}

	fn insert_clips(
		&mut self,
		blend_node: &AnimationNodeIndex,
		clip: AnimationClips,
	) -> AnimationClips<AnimationNodeIndex> {
		match &clip {
			AnimationClips::Single(clip) => {
				let index = self.add_clip(clip.clone(), 1., *blend_node);

				AnimationClips::Single(index)
			}
			AnimationClips::Directional(clips) => {
				let blend_node = self.add_blend(1., *blend_node);

				let Directional {
					forward,
					backward,
					left,
					right,
				} = clips;

				AnimationClips::Directional(Directional {
					forward: self.add_clip(forward.clone(), 0., blend_node),
					backward: self.add_clip(backward.clone(), 0., blend_node),
					left: self.add_clip(left.clone(), 0., blend_node),
					right: self.add_clip(right.clone(), 0., blend_node),
				})
			}
			AnimationClips::PitchedForward(clips) => {
				let blend_node = self.add_blend(1., *blend_node);

				let PitchedForward {
					neutral,
					up: (up_pitch, up),
					down: (down_pitch, down),
				} = clips;

				AnimationClips::PitchedForward(PitchedForward {
					neutral: self.add_clip(neutral.clone(), 0., blend_node),
					up: (*up_pitch, self.add_clip(up.clone(), 0., blend_node)),
					down: (*down_pitch, self.add_clip(down.clone(), 0., blend_node)),
				})
			}
		}
	}
}

trait AnimationGraphTrait {
	fn add_clip(
		&mut self,
		clip: Handle<AnimationClip>,
		weight: f32,
		parent: AnimationNodeIndex,
	) -> AnimationNodeIndex;
	fn add_blend(&mut self, weight: f32, parent: AnimationNodeIndex) -> AnimationNodeIndex;
	fn add_additive_blend(&mut self, weight: f32, parent: AnimationNodeIndex)
	-> AnimationNodeIndex;
	fn root(&self) -> AnimationNodeIndex;
}

impl AnimationGraphTrait for AnimationGraph {
	fn add_clip(
		&mut self,
		clip: Handle<AnimationClip>,
		weight: f32,
		parent: AnimationNodeIndex,
	) -> AnimationNodeIndex {
		self.add_clip(clip, weight, parent)
	}

	fn add_blend(&mut self, weight: f32, parent: AnimationNodeIndex) -> AnimationNodeIndex {
		self.add_blend(weight, parent)
	}

	fn add_additive_blend(
		&mut self,
		weight: f32,
		parent: AnimationNodeIndex,
	) -> AnimationNodeIndex {
		self.add_additive_blend(weight, parent)
	}

	fn root(&self) -> AnimationNodeIndex {
		self.root
	}
}

pub(crate) trait GetNodeMut {
	fn get_node_mut(&mut self, animation: AnimationNodeIndex) -> Option<&mut AnimationGraphNode>;
}

impl GetNodeMut for AnimationGraph {
	fn get_node_mut(&mut self, animation: AnimationNodeIndex) -> Option<&mut AnimationGraphNode> {
		self.get_mut(animation)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::handles_animations::{ForwardPitch, PitchedForward};
	use mockall::{mock, predicate::eq};
	use testing::new_handle;

	macro_rules! setup_graph {
		($setup:expr) => {
			struct _Graph {
				mock: Mock_Graph,
			}

			// used to allow rust to self infer the setup argument type
			fn setup(func: impl Fn(&mut Mock_Graph)) -> impl Fn(&mut Mock_Graph) {
				func
			}

			impl Default for _Graph {
				fn default() -> Self {
					let mut mock = Mock_Graph::default();

					let setup = setup($setup);
					setup(&mut mock);

					// set defaults for everything not set up in setup
					mock.expect_add_clip()
						.return_const(AnimationNodeIndex::new(666));
					mock.expect_add_blend()
						.return_const(AnimationNodeIndex::new(666));
					mock.expect_add_additive_blend()
						.return_const(AnimationNodeIndex::new(666));
					mock.expect_root()
						.return_const(AnimationNodeIndex::new(666));
					Self { mock }
				}
			}

			impl AnimationGraphTrait for _Graph {
				fn add_clip(&mut self, clip: Handle<AnimationClip>, weight: f32, parent: AnimationNodeIndex) -> AnimationNodeIndex {
					self.mock.add_clip(clip, weight, parent)
				}

				fn add_blend(&mut self, weight: f32, parent: AnimationNodeIndex) -> AnimationNodeIndex {
					self.mock.add_blend(weight, parent)
				}

				fn add_additive_blend(&mut self, weight: f32, parent: AnimationNodeIndex) -> AnimationNodeIndex {
					self.mock.add_additive_blend(weight, parent)
				}

				fn root(&self) -> AnimationNodeIndex {
					self.mock.root()
				}
			}

			mock! {
				_Graph {}
				impl AnimationGraphTrait for _Graph {
					fn add_clip(&mut self, clip: Handle<AnimationClip>, weight: f32, parent: AnimationNodeIndex) -> AnimationNodeIndex;
					fn add_blend(&mut self, weight: f32, parent: AnimationNodeIndex) -> AnimationNodeIndex;
					fn add_additive_blend(&mut self, weight: f32, parent: AnimationNodeIndex) -> AnimationNodeIndex;
					fn root(&self) -> AnimationNodeIndex;
				}
			}

		}
	}

	#[test]
	fn build_top_blend_node_for_mask_setup() {
		setup_graph!(|mock| {
			mock.expect_root().return_const(AnimationNodeIndex::new(42));
			mock.expect_add_additive_blend()
				.times(1)
				.with(eq(1.), eq(AnimationNodeIndex::new(42)))
				.return_const(AnimationNodeIndex::new(111));
		});
		let (_, blend_node) = _Graph::with_buffer();

		assert_eq!(AnimationNodeIndex::new(111), blend_node);
	}

	mod single_animation_asset {
		use super::*;
		use std::sync::LazyLock;

		#[test]
		fn add_loaded_clip_to_graph() {
			static HANDLE: LazyLock<Handle<AnimationClip>> = LazyLock::new(new_handle);
			setup_graph!(|mock| {
				mock.expect_add_clip()
					.times(1)
					.with(eq(HANDLE.clone()), eq(1.), eq(AnimationNodeIndex::new(11)))
					.return_const(AnimationNodeIndex::default());
			});

			let (mut graph, _) = _Graph::with_buffer();

			graph.insert_clips(
				&AnimationNodeIndex::new(11),
				AnimationClips::Single(HANDLE.clone()),
			);
		}

		#[test]
		fn return_animation_node_index() {
			setup_graph!(|mock| {
				mock.expect_add_clip()
					.return_const(AnimationNodeIndex::new(111));
			});
			let (mut graph, buffer) = _Graph::with_buffer();

			let index = graph.insert_clips(&buffer, AnimationClips::Single(new_handle()));

			assert_eq!(AnimationClips::Single(AnimationNodeIndex::new(111)), index);
		}
	}

	mod directional_animation_asset {
		use super::*;
		use std::sync::LazyLock;

		#[test]
		fn add_loaded_clips_to_graph() {
			static FORWARD: LazyLock<Handle<AnimationClip>> = LazyLock::new(new_handle);
			static BACKWARD: LazyLock<Handle<AnimationClip>> = LazyLock::new(new_handle);
			static LEFT: LazyLock<Handle<AnimationClip>> = LazyLock::new(new_handle);
			static RIGHT: LazyLock<Handle<AnimationClip>> = LazyLock::new(new_handle);
			setup_graph!(|mock| {
				mock.expect_add_blend()
					.times(1)
					.with(eq(1.), eq(AnimationNodeIndex::new(11)))
					.return_const(AnimationNodeIndex::new(4242));
				mock.expect_add_clip()
					.times(1)
					.with(
						eq(FORWARD.clone()),
						eq(0.),
						eq(AnimationNodeIndex::new(4242)),
					)
					.return_const(AnimationNodeIndex::default());
				mock.expect_add_clip()
					.times(1)
					.with(
						eq(BACKWARD.clone()),
						eq(0.),
						eq(AnimationNodeIndex::new(4242)),
					)
					.return_const(AnimationNodeIndex::default());
				mock.expect_add_clip()
					.times(1)
					.with(eq(LEFT.clone()), eq(0.), eq(AnimationNodeIndex::new(4242)))
					.return_const(AnimationNodeIndex::default());
				mock.expect_add_clip()
					.times(1)
					.with(eq(RIGHT.clone()), eq(0.), eq(AnimationNodeIndex::new(4242)))
					.return_const(AnimationNodeIndex::default());
			});

			let (mut graph, _) = _Graph::with_buffer();

			graph.insert_clips(
				&AnimationNodeIndex::new(11),
				AnimationClips::Directional(Directional {
					forward: FORWARD.clone(),
					backward: BACKWARD.clone(),
					left: LEFT.clone(),
					right: RIGHT.clone(),
				}),
			);
		}

		#[test]
		fn return_animation_node_indices() {
			static FORWARD: LazyLock<Handle<AnimationClip>> = LazyLock::new(new_handle);
			static BACKWARD: LazyLock<Handle<AnimationClip>> = LazyLock::new(new_handle);
			static LEFT: LazyLock<Handle<AnimationClip>> = LazyLock::new(new_handle);
			static RIGHT: LazyLock<Handle<AnimationClip>> = LazyLock::new(new_handle);
			setup_graph!(|mock| {
				mock.expect_add_blend()
					.with(eq(1.), eq(AnimationNodeIndex::new(11)))
					.return_const(AnimationNodeIndex::new(4242));
				mock.expect_add_clip()
					.with(
						eq(FORWARD.clone()),
						eq(0.),
						eq(AnimationNodeIndex::new(4242)),
					)
					.return_const(AnimationNodeIndex::new(1));
				mock.expect_add_clip()
					.with(
						eq(BACKWARD.clone()),
						eq(0.),
						eq(AnimationNodeIndex::new(4242)),
					)
					.return_const(AnimationNodeIndex::new(2));
				mock.expect_add_clip()
					.with(eq(LEFT.clone()), eq(0.), eq(AnimationNodeIndex::new(4242)))
					.return_const(AnimationNodeIndex::new(3));
				mock.expect_add_clip()
					.with(eq(RIGHT.clone()), eq(0.), eq(AnimationNodeIndex::new(4242)))
					.return_const(AnimationNodeIndex::new(4));
			});
			let (mut graph, _) = _Graph::with_buffer();

			let index = graph.insert_clips(
				&AnimationNodeIndex::new(11),
				AnimationClips::Directional(Directional {
					forward: FORWARD.clone(),
					backward: BACKWARD.clone(),
					left: LEFT.clone(),
					right: RIGHT.clone(),
				}),
			);

			assert_eq!(
				AnimationClips::Directional(Directional {
					forward: AnimationNodeIndex::new(1),
					backward: AnimationNodeIndex::new(2),
					left: AnimationNodeIndex::new(3),
					right: AnimationNodeIndex::new(4)
				}),
				index
			);
		}
	}

	mod pitched_animation_asset {
		#![allow(clippy::unwrap_used)]
		use common::forward_pitch;

		use super::*;
		use std::sync::LazyLock;

		#[test]
		fn add_loaded_clips_to_graph() {
			static NEUTRAL: LazyLock<Handle<AnimationClip>> = LazyLock::new(new_handle);
			static UP: LazyLock<Handle<AnimationClip>> = LazyLock::new(new_handle);
			static DOWN: LazyLock<Handle<AnimationClip>> = LazyLock::new(new_handle);
			setup_graph!(|mock| {
				mock.expect_add_blend()
					.times(1)
					.with(eq(1.), eq(AnimationNodeIndex::new(11)))
					.return_const(AnimationNodeIndex::new(4242));
				mock.expect_add_clip()
					.times(1)
					.with(
						eq(NEUTRAL.clone()),
						eq(0.),
						eq(AnimationNodeIndex::new(4242)),
					)
					.return_const(AnimationNodeIndex::default());
				mock.expect_add_clip()
					.times(1)
					.with(eq(UP.clone()), eq(0.), eq(AnimationNodeIndex::new(4242)))
					.return_const(AnimationNodeIndex::default());
				mock.expect_add_clip()
					.times(1)
					.with(eq(DOWN.clone()), eq(0.), eq(AnimationNodeIndex::new(4242)))
					.return_const(AnimationNodeIndex::default());
			});

			let (mut graph, _) = _Graph::with_buffer();

			graph.insert_clips(
				&AnimationNodeIndex::new(11),
				AnimationClips::PitchedForward(PitchedForward {
					neutral: NEUTRAL.clone(),
					up: (ForwardPitch::MAX, UP.clone()),
					down: (ForwardPitch::MAX, DOWN.clone()),
				}),
			);
		}

		#[test]
		fn return_animation_node_indices() {
			static NEUTRAL: LazyLock<Handle<AnimationClip>> = LazyLock::new(new_handle);
			static UP: LazyLock<Handle<AnimationClip>> = LazyLock::new(new_handle);
			static DOWN: LazyLock<Handle<AnimationClip>> = LazyLock::new(new_handle);
			setup_graph!(|mock| {
				mock.expect_add_blend()
					.with(eq(1.), eq(AnimationNodeIndex::new(11)))
					.return_const(AnimationNodeIndex::new(4242));
				mock.expect_add_clip()
					.with(
						eq(NEUTRAL.clone()),
						eq(0.),
						eq(AnimationNodeIndex::new(4242)),
					)
					.return_const(AnimationNodeIndex::new(1));
				mock.expect_add_clip()
					.with(eq(UP.clone()), eq(0.), eq(AnimationNodeIndex::new(4242)))
					.return_const(AnimationNodeIndex::new(2));
				mock.expect_add_clip()
					.with(eq(DOWN.clone()), eq(0.), eq(AnimationNodeIndex::new(4242)))
					.return_const(AnimationNodeIndex::new(3));
			});
			let (mut graph, _) = _Graph::with_buffer();

			let index = graph.insert_clips(
				&AnimationNodeIndex::new(11),
				AnimationClips::PitchedForward(PitchedForward {
					neutral: NEUTRAL.clone(),
					up: (forward_pitch!(0.2), UP.clone()),
					down: (forward_pitch!(0.1), DOWN.clone()),
				}),
			);

			assert_eq!(
				AnimationClips::PitchedForward(PitchedForward {
					neutral: AnimationNodeIndex::new(1),
					up: (forward_pitch!(0.2), AnimationNodeIndex::new(2)),
					down: (forward_pitch!(0.1), AnimationNodeIndex::new(3)),
				}),
				index
			);
		}
	}
}
