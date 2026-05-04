use crate::{
	assets::agent_config::AgentConfigAsset,
	components::{agent::ApplyAgentAnimations, agent_config::AgentConfig},
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	components::gltf::GltfLookup,
	errors::{ErrorData, Level},
	traits::{
		accessors::get::{GetContextMut, TryApplyOn},
		handles_animations::{
			Animation,
			AnimationClips,
			AnimationKey,
			RegisterAnimations,
			WithoutAnimations,
		},
	},
	zyheeda_commands::ZyheedaCommands,
};
use std::{
	collections::{HashMap, HashSet},
	fmt::Display,
};

impl ApplyAgentAnimations {
	pub(crate) fn system<TAnimations>(
		mut commands: ZyheedaCommands,
		mut animations: StaticSystemParam<TAnimations>,
		configs: Res<Assets<AgentConfigAsset>>,
		models: Res<Assets<Gltf>>,
		agents: Query<(Entity, &AgentConfig, &GltfLookup), With<Self>>,
	) -> Result<(), Vec<AnimationsMissing>>
	where
		TAnimations: for<'c> GetContextMut<WithoutAnimations, TContext<'c>: RegisterAnimations>,
	{
		let mut errors = vec![];

		for (entity, AgentConfig { config_handle }, GltfLookup(gltf)) in agents {
			let key = WithoutAnimations { entity };
			let Some(mut ctx) = TAnimations::get_context_mut(&mut animations, key) else {
				continue;
			};

			let Some(config) = configs.get(config_handle) else {
				continue;
			};

			let Some(gltf) = models.get(gltf) else {
				continue;
			};

			let mut missing = HashSet::new();
			let animations = config
				.animations
				.iter()
				.filter_map(get_clips(gltf, &mut missing))
				.collect::<HashMap<_, _>>();

			if !missing.is_empty() {
				errors.push(AnimationsMissing {
					missing,
					available: gltf.named_animations.keys().cloned().collect(),
				});
			}

			ctx.register_animations(&animations, &config.animation_mask_groups);
			commands.try_apply_on(&entity, |mut e| {
				e.try_remove::<Self>();
			});
		}

		if !errors.is_empty() {
			return Err(errors);
		}

		Ok(())
	}
}

type AnimationKeyAndNames<'a> = (&'a AnimationKey, &'a Animation<AnimationClips<String>>);
type AnimationKeyAndClips = (AnimationKey, Animation);

fn get_clips(
	gltf: &Gltf,
	missing: &mut HashSet<String>,
) -> impl FnMut(AnimationKeyAndNames) -> Option<AnimationKeyAndClips> {
	|(key, animation)| {
		let get_clips = |name: String| match gltf.named_animations.get(name.as_str()) {
			Some(clip) => Some(clip.clone()),
			None => {
				missing.insert(name);
				None
			}
		};

		Some((
			*key,
			Animation {
				clips: animation.clips.clone().try_map_option(get_clips)?,
				play_mode: animation.play_mode,
				mask_groups: animation.mask_groups,
			},
		))
	}
}

#[derive(Debug, PartialEq)]
pub(crate) struct AnimationsMissing {
	missing: HashSet<String>,
	available: HashSet<Box<str>>,
}

impl Display for AnimationsMissing {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"Missing animation: [{}]. Available animations: [{}].",
			Vec::from_iter(self.missing.iter().cloned()).join(", "),
			Vec::from_iter(self.available.iter().cloned()).join(", ")
		)
	}
}

impl ErrorData for AnimationsMissing {
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl Display {
		"Animations Missing"
	}

	fn into_details(self) -> impl Display {
		self
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{assets::agent_config::AgentConfigAsset, components::agent_config::AgentConfig};
	use common::{
		bit_mask_index,
		tools::bone_name::BoneName,
		traits::handles_animations::{
			AffectedAnimationBones,
			Animation,
			AnimationClips,
			AnimationKey,
			AnimationMaskBits,
		},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::{collections::HashMap, sync::LazyLock};
	use testing::{NestedMocks, SingleThreadedApp, new_handle};

	#[derive(Component, NestedMocks)]
	struct _Animations {
		mock: Mock_Animations,
	}

	impl Default for _Animations {
		fn default() -> Self {
			Self::new().with_mock(|mock| {
				mock.expect_register_animations().return_const(());
			})
		}
	}

	#[automock]
	impl RegisterAnimations for _Animations {
		fn register_animations(
			&mut self,
			animations: &HashMap<AnimationKey, Animation>,
			animation_mask_groups: &HashMap<AnimationMaskBits, AffectedAnimationBones>,
		) {
			self.mock
				.register_animations(animations, animation_mask_groups);
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(Result<(), Vec<AnimationsMissing>>);

	fn setup<const C: usize, const M: usize>(
		configs: [(&Handle<AgentConfigAsset>, AgentConfigAsset); C],
		models: [(&Handle<Gltf>, Gltf); M],
	) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut config_assets = Assets::default();
		let mut model_assets = Assets::default();

		for (id, asset) in configs {
			_ = config_assets.insert(id, asset);
		}

		for (id, asset) in models {
			_ = model_assets.insert(id, asset);
		}

		app.insert_resource(config_assets);
		app.insert_resource(model_assets);
		app.add_systems(
			Update,
			ApplyAgentAnimations::system::<Query<&mut _Animations>>.pipe(
				|In(r), mut c: Commands| {
					c.insert_resource(_Result(r));
				},
			),
		);

		app
	}

	fn gltf(
		named_animations: impl IntoIterator<Item = (&'static str, Handle<AnimationClip>)>,
	) -> Gltf {
		Gltf {
			scenes: [].into(),
			named_scenes: [].into(),
			meshes: [].into(),
			named_meshes: [].into(),
			materials: [].into(),
			named_materials: [].into(),
			nodes: [].into(),
			named_nodes: [].into(),
			skins: [].into(),
			named_skins: [].into(),
			default_scene: None,
			animations: [].into(),
			named_animations: named_animations
				.into_iter()
				.map(|(name, clip)| (Box::from(name), clip))
				.collect(),
			source: None,
		}
	}

	#[test]
	fn register_animations() {
		static CLIP: LazyLock<Handle<AnimationClip>> = LazyLock::new(new_handle);
		let config = AgentConfigAsset {
			animations: HashMap::from([(
				AnimationKey::Run,
				Animation {
					clips: AnimationClips::Single("Run".to_owned()),
					..default()
				},
			)]),
			animation_mask_groups: HashMap::from([(
				AnimationMaskBits::zero().with_set(bit_mask_index!(2)),
				AffectedAnimationBones {
					from_root: BoneName::from("root"),
					until_exclusive: [].into(),
				},
			)]),
			..default()
		};
		let config_handle = new_handle();
		let gltf_handle = new_handle();
		let gltf = gltf([("Run", CLIP.clone())]);
		let mut app = setup([(&config_handle, config)], [(&gltf_handle, gltf)]);
		app.world_mut().spawn((
			ApplyAgentAnimations,
			GltfLookup(gltf_handle),
			AgentConfig { config_handle },
			_Animations::new().with_mock(assert_animations_registered),
		));

		app.update();

		fn assert_animations_registered(mock: &mut Mock_Animations) {
			mock.expect_register_animations()
				.once()
				.with(
					eq(HashMap::from([(
						AnimationKey::Run,
						Animation {
							clips: AnimationClips::Single(CLIP.clone()),
							..default()
						},
					)])),
					eq(HashMap::from([(
						AnimationMaskBits::zero().with_set(bit_mask_index!(2)),
						AffectedAnimationBones {
							from_root: BoneName::from("root"),
							until_exclusive: [].into(),
						},
					)])),
				)
				.return_const(());
		}
	}

	#[test]
	fn remove_apply_agent_animations_marker() {
		let config_handle = new_handle();
		let gltf_handle = new_handle();
		let mut app = setup(
			[(&config_handle, AgentConfigAsset::default())],
			[(&gltf_handle, gltf([]))],
		);
		let entity = app
			.world_mut()
			.spawn((
				ApplyAgentAnimations,
				GltfLookup(gltf_handle),
				AgentConfig { config_handle },
				_Animations::default(),
			))
			.id();

		app.update();

		assert_eq!(
			None,
			app.world().entity(entity).get::<ApplyAgentAnimations>()
		);
	}

	#[test]
	fn do_nothing_if_apply_agent_animation_marker_missing() {
		let config_handle = new_handle();
		let gltf_handle = new_handle();
		let mut app = setup(
			[(&config_handle, AgentConfigAsset::default())],
			[(&gltf_handle, gltf([]))],
		);
		app.world_mut().spawn((
			GltfLookup(gltf_handle),
			AgentConfig { config_handle },
			_Animations::new().with_mock(assert_register_not_called),
		));

		app.update();

		fn assert_register_not_called(mock: &mut Mock_Animations) {
			mock.expect_register_animations().never().return_const(());
		}
	}

	#[test]
	fn return_animations_missing() {
		let config = AgentConfigAsset {
			animations: HashMap::from([
				(
					AnimationKey::Walk,
					Animation {
						clips: AnimationClips::Single("Walk".to_owned()),
						..default()
					},
				),
				(
					AnimationKey::Run,
					Animation {
						clips: AnimationClips::Single("Run".to_owned()),
						..default()
					},
				),
				(
					AnimationKey::Idle,
					Animation {
						clips: AnimationClips::Single("Idle".to_owned()),
						..default()
					},
				),
			]),
			animation_mask_groups: HashMap::from([]),
			..default()
		};
		let config_handle = new_handle();
		let gltf_handle = new_handle();
		let gltf = gltf([("Run", new_handle())]);
		let mut app = setup([(&config_handle, config)], [(&gltf_handle, gltf)]);
		app.world_mut().spawn((
			ApplyAgentAnimations,
			GltfLookup(gltf_handle),
			AgentConfig { config_handle },
			_Animations::new().with_mock(|mock| {
				mock.expect_register_animations().return_const(());
			}),
		));

		app.update();

		assert_eq!(
			&_Result(Err(vec![AnimationsMissing {
				missing: HashSet::from(["Walk".to_owned(), "Idle".to_owned()]),
				available: HashSet::from([Box::from("Run")]),
			}])),
			app.world().resource::<_Result>()
		);
	}

	#[test]
	fn return_ok_when_all_animations_found() {
		let config = AgentConfigAsset {
			animations: HashMap::from([
				(
					AnimationKey::Walk,
					Animation {
						clips: AnimationClips::Single("Walk".to_owned()),
						..default()
					},
				),
				(
					AnimationKey::Run,
					Animation {
						clips: AnimationClips::Single("Run".to_owned()),
						..default()
					},
				),
				(
					AnimationKey::Idle,
					Animation {
						clips: AnimationClips::Single("Idle".to_owned()),
						..default()
					},
				),
			]),
			animation_mask_groups: HashMap::from([]),
			..default()
		};
		let config_handle = new_handle();
		let gltf_handle = new_handle();
		let gltf = gltf([
			("Idle", new_handle()),
			("Walk", new_handle()),
			("Run", new_handle()),
		]);
		let mut app = setup([(&config_handle, config)], [(&gltf_handle, gltf)]);
		app.world_mut().spawn((
			ApplyAgentAnimations,
			GltfLookup(gltf_handle),
			AgentConfig { config_handle },
			_Animations::new().with_mock(|mock| {
				mock.expect_register_animations().return_const(());
			}),
		));

		app.update();

		assert_eq!(&_Result(Ok(())), app.world().resource::<_Result>());
	}
}
