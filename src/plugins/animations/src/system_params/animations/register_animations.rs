use crate::{
	components::animation_lookup::AnimationLookup,
	system_params::animations::AnimationsRegisterContextMut,
	traits::InsertClips,
};
use bevy::prelude::*;
use common::traits::{
	handles_animations::{
		AffectedAnimationBones,
		Animation,
		AnimationClips,
		AnimationKey,
		AnimationMaskBits,
		RegisterAnimations,
	},
	wrap_handle::WrapHandle,
};
use std::collections::HashMap;

impl<TGraph> RegisterAnimations for AnimationsRegisterContextMut<'_, TGraph>
where
	TGraph: Asset + WrapHandle + InsertClips<AnimationClips<AnimationNodeIndex>>,
{
	fn register_animations(
		&mut self,
		animations: &HashMap<AnimationKey, Animation>,
		animation_mask_groups: &HashMap<AnimationMaskBits, AffectedAnimationBones>,
	) {
		let (mut graph, buffer) = TGraph::with_buffer();
		let insert_into_graph = |animation: &Animation| Animation {
			clips: graph.insert_clips(&buffer, animation.clips.clone()),
			play_mode: animation.play_mode,
			mask_groups: animation.mask_groups,
		};

		self.entity.try_insert((
			AnimationLookup {
				animation_mask_groups: animation_mask_groups.clone(),
				animations: map_animations(animations, insert_into_graph),
			},
			TGraph::wrap_handle(self.graphs.add(graph)),
		));
	}
}

fn map_animations(
	animations: &HashMap<AnimationKey, Animation>,
	mut f: impl FnMut(&Animation) -> Animation<AnimationClips<AnimationNodeIndex>>,
) -> HashMap<AnimationKey, Animation<AnimationClips<AnimationNodeIndex>>> {
	animations
		.iter()
		.map(|(key, animation)| (*key, f(animation)))
		.collect()
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
			accessors::get::TryGetContextMut,
			handles_animations::{
				AffectedAnimationBones,
				AnimationClips,
				PlayMode,
				SkillAnimation,
				WithoutAnimations,
			},
			wrap_handle::GetHandle,
		},
	};
	use std::{collections::HashSet, sync::LazyLock};
	use testing::{SingleThreadedApp, new_handle};

	static CLIP_A: LazyLock<Handle<AnimationClip>> = LazyLock::new(new_handle);
	static CLIP_B: LazyLock<Handle<AnimationClip>> = LazyLock::new(new_handle);
	static CLIP_C: LazyLock<Handle<AnimationClip>> = LazyLock::new(new_handle);

	#[derive(Asset, TypePath, Debug, PartialEq)]
	struct _Graph(Vec<Handle<AnimationClip>>);

	impl InsertClips<AnimationClips<AnimationNodeIndex>> for _Graph {
		type TBuffer = ();

		fn with_buffer() -> (Self, ()) {
			(Self(vec![]), ())
		}

		fn insert_clips(
			&mut self,
			_: &(),
			animations: AnimationClips,
		) -> AnimationClips<AnimationNodeIndex> {
			animations.map(|clip| {
				self.0.push(clip.clone());
				AnimationNodeIndex::new(match clip {
					c if c == *CLIP_A => 0,
					c if c == *CLIP_B => 1,
					c if c == *CLIP_C => 2,
					_ => 666,
				})
			})
		}
	}

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

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(Assets::<_Graph>::default());

		app
	}

	#[test]
	fn add_animation_graph() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut p: AnimationsParamMut<_Graph>| {
				let key = WithoutAnimations { entity };
				let mut ctx = AnimationsParamMut::try_get_context_mut(&mut p, key).unwrap();
				ctx.register_animations(&HashMap::default(), &HashMap::default());
			})?;

		assert!(app.world().entity(entity).contains::<_GraphComponent>());
		Ok(())
	}

	#[test]
	fn add_animation_dispatch() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut p: AnimationsParamMut<_Graph>| {
				let key = WithoutAnimations { entity };
				let mut ctx = AnimationsParamMut::try_get_context_mut(&mut p, key).unwrap();
				ctx.register_animations(&HashMap::default(), &HashMap::default());
			})?;

		assert_eq!(
			Some(&AnimationDispatch::default()),
			app.world().entity(entity).get::<AnimationDispatch>()
		);
		Ok(())
	}

	#[test]
	fn insert_animation_graph_asset() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut p: AnimationsParamMut<_Graph>| {
				let key = WithoutAnimations { entity };
				let mut ctx = AnimationsParamMut::try_get_context_mut(&mut p, key).unwrap();
				let a = Animation {
					clips: AnimationClips::Single(CLIP_A.clone()),
					play_mode: PlayMode::Repeat,
					mask_groups: AnimationMaskBits::zero().with_set(bit_mask_index!(0)),
				};
				let b = Animation {
					clips: AnimationClips::Single(CLIP_B.clone()),
					play_mode: PlayMode::Repeat,
					mask_groups: AnimationMaskBits::zero().with_set(bit_mask_index!(1)),
				};
				let c = Animation {
					clips: AnimationClips::Single(CLIP_C.clone()),
					play_mode: PlayMode::Replay,
					mask_groups: AnimationMaskBits::zero().with_set(bit_mask_index!(2)),
				};

				ctx.register_animations(
					&HashMap::from([
						(AnimationKey::Idle, a),
						(AnimationKey::Walk, b),
						(
							AnimationKey::Skill {
								slot: SlotKey(42),
								animation: SkillAnimation::Aim,
							},
							c,
						),
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
			Some(HashSet::from([
				CLIP_A.clone(),
				CLIP_B.clone(),
				CLIP_C.clone()
			])),
			app.world()
				.resource::<Assets<_Graph>>()
				.iter()
				.next()
				.map(|(_, g)| g.0.iter().cloned().collect::<HashSet<_>>()),
		);
		Ok(())
	}

	#[test]
	fn add_animation_lookup() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut p: AnimationsParamMut<_Graph>| {
				let key = WithoutAnimations { entity };
				let mut ctx = AnimationsParamMut::try_get_context_mut(&mut p, key).unwrap();
				let a = Animation {
					clips: AnimationClips::Single(CLIP_A.clone()),
					play_mode: PlayMode::Repeat,
					mask_groups: AnimationMaskBits::zero().with_set(bit_mask_index!(0)),
				};
				let b = Animation {
					clips: AnimationClips::Single(CLIP_B.clone()),
					play_mode: PlayMode::Repeat,
					mask_groups: AnimationMaskBits::zero().with_set(bit_mask_index!(1)),
				};
				let c = Animation {
					clips: AnimationClips::Single(CLIP_C.clone()),
					play_mode: PlayMode::Replay,
					mask_groups: AnimationMaskBits::zero().with_set(bit_mask_index!(2)),
				};

				ctx.register_animations(
					&HashMap::from([
						(AnimationKey::Idle, a),
						(AnimationKey::Walk, b),
						(
							AnimationKey::Skill {
								slot: SlotKey(42),
								animation: SkillAnimation::Aim,
							},
							c,
						),
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
						Animation {
							clips: AnimationClips::Single(AnimationNodeIndex::new(0)),
							play_mode: PlayMode::Repeat,
							mask_groups: AnimationMaskBits::zero().with_set(bit_mask_index!(0)),
						},
					),
					(
						AnimationKey::Walk,
						Animation {
							clips: AnimationClips::Single(AnimationNodeIndex::new(1)),
							play_mode: PlayMode::Repeat,
							mask_groups: AnimationMaskBits::zero().with_set(bit_mask_index!(1)),
						},
					),
					(
						AnimationKey::Skill {
							slot: SlotKey(42),
							animation: SkillAnimation::Aim,
						},
						Animation {
							clips: AnimationClips::Single(AnimationNodeIndex::new(2)),
							play_mode: PlayMode::Replay,
							mask_groups: AnimationMaskBits::zero().with_set(bit_mask_index!(2)),
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
