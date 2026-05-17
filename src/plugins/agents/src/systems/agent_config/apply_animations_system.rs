use crate::{
	assets::agent_meta::AgentMeta,
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
			AnimationKey,
			AnimationName,
			AnimationNames,
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
		configs: Res<Assets<AgentMeta>>,
		models: Res<Assets<Gltf>>,
		agents: Query<(Entity, &AgentConfig, Option<&GltfLookup>), With<Self>>,
	) -> Result<(), Vec<RegisterAnimationsError>>
	where
		TAnimations: for<'c> GetContextMut<WithoutAnimations, TContext<'c>: RegisterAnimations>,
	{
		let mut errors = vec![];

		for (entity, AgentConfig { config_handle }, gltf) in agents {
			let key = WithoutAnimations { entity };
			let Some(mut ctx) = TAnimations::get_context_mut(&mut animations, key) else {
				continue;
			};

			let Some(config) = configs.get(config_handle) else {
				continue;
			};

			let animations = match gltf {
				Some(GltfLookup(gltf)) => {
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
						errors.push(RegisterAnimationsError::MissingAnimations {
							entity,
							missing,
							available: gltf.named_animations.keys().cloned().collect(),
						});
					}
					animations
				}
				None => {
					if !config.animations.is_empty() {
						errors.push(RegisterAnimationsError::GltfLookupMissing(entity));
					}
					HashMap::new()
				}
			};

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

type AnimationKeyAndNames<'a> = (&'a AnimationKey, &'a Animation<AnimationNames>);
type AnimationKeyAndClips = (AnimationKey, Animation);

fn get_clips(
	gltf: &Gltf,
	missing: &mut HashSet<AnimationName>,
) -> impl FnMut(AnimationKeyAndNames) -> Option<AnimationKeyAndClips> {
	|(key, animation)| {
		let get_clips = |name: AnimationName| match gltf.named_animations.get(&*name) {
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
pub(crate) enum RegisterAnimationsError {
	MissingAnimations {
		entity: Entity,
		missing: HashSet<AnimationName>,
		available: HashSet<Box<str>>,
	},
	GltfLookupMissing(Entity),
}

impl Display for RegisterAnimationsError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			RegisterAnimationsError::MissingAnimations {
				entity,
				missing,
				available,
			} => {
				write!(
					f,
					"{}: Missing animation: [{}]. Available animations: [{}].",
					entity,
					Vec::from_iter(missing.iter().cloned()).join(", "),
					Vec::from_iter(available.iter().cloned()).join(", ")
				)
			}
			RegisterAnimationsError::GltfLookupMissing(entity) => {
				write!(
					f,
					"{entity}: missing gltf lookup component, which is required if the agent has animations"
				)
			}
		}
	}
}

impl ErrorData for RegisterAnimationsError {
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl Display {
		"Register Animations Error"
	}

	fn into_details(self) -> impl Display {
		self
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{assets::agent_meta::AgentMeta, components::agent_config::AgentConfig};
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
	struct _Result(Result<(), Vec<RegisterAnimationsError>>);

	fn setup<const C: usize, const M: usize>(
		configs: [(&Handle<AgentMeta>, AgentMeta); C],
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
		let config = AgentMeta {
			animations: HashMap::from([(
				AnimationKey::Run,
				Animation {
					clips: AnimationClips::Single(AnimationName::from("Run")),
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
	fn register_animations_delayed() {
		static CLIP: LazyLock<Handle<AnimationClip>> = LazyLock::new(new_handle);
		let config = AgentMeta {
			animations: HashMap::from([(
				AnimationKey::Run,
				Animation {
					clips: AnimationClips::Single(AnimationName::from("Run")),
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
		let mut app = setup([(&config_handle, config)], []);
		app.world_mut().spawn((
			ApplyAgentAnimations,
			GltfLookup(gltf_handle.clone()),
			AgentConfig { config_handle },
			_Animations::new().with_mock(assert_animations_registered),
		));

		app.update();
		_ = app
			.world_mut()
			.resource_mut::<Assets<Gltf>>()
			.insert(&gltf_handle, gltf);
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
	fn no_gltf_lookup_required_when_animations_empty() {
		let config_handle = new_handle();
		let config = AgentMeta {
			animations: HashMap::from([]),
			animation_mask_groups: HashMap::from([(
				AnimationMaskBits::zero().with_set(bit_mask_index!(2)),
				AffectedAnimationBones {
					from_root: BoneName::from("root"),
					until_exclusive: [].into(),
				},
			)]),
			..default()
		};
		let mut app = setup([(&config_handle, config)], []);
		app.world_mut().spawn((
			ApplyAgentAnimations,
			AgentConfig { config_handle },
			_Animations::new().with_mock(assert_animations_registered),
		));

		app.update();

		fn assert_animations_registered(mock: &mut Mock_Animations) {
			mock.expect_register_animations()
				.once()
				.with(
					eq(HashMap::from([])),
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
	fn set_empty_animations_when_no_gltf_lookup() {
		let config_handle = new_handle();
		let config = AgentMeta {
			animations: HashMap::from([(
				AnimationKey::Run,
				Animation {
					clips: AnimationClips::Single(AnimationName::from("Run")),
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
		let mut app = setup([(&config_handle, config)], []);
		app.world_mut().spawn((
			ApplyAgentAnimations,
			AgentConfig { config_handle },
			_Animations::new().with_mock(assert_animations_registered),
		));

		app.update();

		fn assert_animations_registered(mock: &mut Mock_Animations) {
			mock.expect_register_animations()
				.once()
				.with(
					eq(HashMap::from([])),
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
			[(&config_handle, AgentMeta::default())],
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
			[(&config_handle, AgentMeta::default())],
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
		let config = AgentMeta {
			animations: HashMap::from([
				(
					AnimationKey::Walk,
					Animation {
						clips: AnimationClips::Single(AnimationName::from("Walk")),
						..default()
					},
				),
				(
					AnimationKey::Run,
					Animation {
						clips: AnimationClips::Single(AnimationName::from("Run")),
						..default()
					},
				),
				(
					AnimationKey::Idle,
					Animation {
						clips: AnimationClips::Single(AnimationName::from("Idle")),
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
		let entity = app
			.world_mut()
			.spawn((
				ApplyAgentAnimations,
				GltfLookup(gltf_handle),
				AgentConfig { config_handle },
				_Animations::new().with_mock(|mock| {
					mock.expect_register_animations().return_const(());
				}),
			))
			.id();

		app.update();

		assert_eq!(
			&_Result(Err(vec![RegisterAnimationsError::MissingAnimations {
				entity,
				missing: HashSet::from([AnimationName::from("Walk"), AnimationName::from("Idle")]),
				available: HashSet::from([Box::from("Run")]),
			}])),
			app.world().resource::<_Result>()
		);
	}

	#[test]
	fn return_gltf_lookup_missing() {
		let config = AgentMeta {
			animations: HashMap::from([(
				AnimationKey::Run,
				Animation {
					clips: AnimationClips::Single(AnimationName::from("Run")),
					..default()
				},
			)]),
			animation_mask_groups: HashMap::from([]),
			..default()
		};
		let config_handle = new_handle();
		let mut app = setup([(&config_handle, config)], []);
		let entity = app
			.world_mut()
			.spawn((
				ApplyAgentAnimations,
				AgentConfig { config_handle },
				_Animations::new().with_mock(|mock| {
					mock.expect_register_animations().return_const(());
				}),
			))
			.id();

		app.update();

		assert_eq!(
			&_Result(Err(vec![RegisterAnimationsError::GltfLookupMissing(
				entity
			)])),
			app.world().resource::<_Result>()
		);
	}

	#[test]
	fn return_ok_when_all_animations_found() {
		let config = AgentMeta {
			animations: HashMap::from([
				(
					AnimationKey::Walk,
					Animation {
						clips: AnimationClips::Single(AnimationName::from("Walk")),
						..default()
					},
				),
				(
					AnimationKey::Run,
					Animation {
						clips: AnimationClips::Single(AnimationName::from("Run")),
						..default()
					},
				),
				(
					AnimationKey::Idle,
					Animation {
						clips: AnimationClips::Single(AnimationName::from("Idle")),
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

	#[test]
	fn return_ok_when_no_gltf_lookup_and_no_animations() {
		let config = AgentMeta {
			animations: HashMap::from([]),
			animation_mask_groups: HashMap::from([]),
			..default()
		};
		let config_handle = new_handle();
		let mut app = setup([(&config_handle, config)], []);
		app.world_mut().spawn((
			ApplyAgentAnimations,
			AgentConfig { config_handle },
			_Animations::new().with_mock(|mock| {
				mock.expect_register_animations().return_const(());
			}),
		));

		app.update();

		assert_eq!(&_Result(Ok(())), app.world().resource::<_Result>());
	}
}
