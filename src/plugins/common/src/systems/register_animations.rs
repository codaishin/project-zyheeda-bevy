use crate::{
	components::gltf::GltfLookup,
	errors::{ErrorData, Level, Unreachable},
	traits::{
		accessors::get::{GetContextMut, TryApplyOn, View},
		handles_animations::{
			AffectedAnimationBones,
			Animation,
			AnimationKey,
			AnimationMaskBits,
			AnimationName,
			AnimationNames,
			RegisterAnimations,
			WithoutAnimations,
		},
	},
	zyheeda_commands::ZyheedaCommands,
};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use std::{
	any::type_name,
	collections::{HashMap, HashSet},
	fmt::Display,
	marker::PhantomData,
};

impl<T> RegisterAnimationsSystem for T where T: AnimationsMarker {}

pub trait RegisterAnimationsSystem: AnimationsMarker + Sized {
	/// Reusable system to register animations and animation masks when `Self` is present on
	/// an entity for which animations and masks have not been registered yet. After registration
	/// `Self` is removed.
	///
	/// Returns a collection of per entity errors.
	#[allow(clippy::type_complexity)]
	fn register_animations_system<TAnimations>(
		mut commands: ZyheedaCommands,
		mut animations: StaticSystemParam<TAnimations>,
		configs: Res<Assets<Self::TConfig>>,
		models: Res<Assets<Gltf>>,
		agents: Query<(Entity, &Self::TConfigComponent, Option<&GltfLookup>), With<Self>>,
	) -> Result<(), Vec<RegisterAnimationsError<Self>>>
	where
		TAnimations: SystemParam
			+ for<'c> GetContextMut<WithoutAnimations, TContext<'c>: RegisterAnimations>,
	{
		let mut errors = vec![];

		for (entity, config, gltf) in agents {
			let key = WithoutAnimations { entity };
			let Some(mut ctx) = TAnimations::get_context_mut(&mut animations, key) else {
				continue;
			};

			let Some(config) = configs.get(config.view()) else {
				continue;
			};

			let (animations, masks) = match gltf {
				Some(GltfLookup(gltf)) => {
					let Some(gltf) = models.get(gltf) else {
						continue;
					};

					let mut missing = HashSet::new();
					let animations = config
						.animations()
						.filter_map(get_clips(gltf, &mut missing))
						.collect::<HashMap<_, _>>();
					if !missing.is_empty() {
						errors.push(RegisterAnimationsError::MissingAnimations {
							entity,
							missing,
							available: gltf.named_animations.keys().cloned().collect(),
						});
					}
					(animations, config.masks().collect())
				}
				None => {
					if config.animations().len() != 0 {
						errors.push(RegisterAnimationsError::GltfLookupMissing { entity });
					}
					(HashMap::new(), HashMap::new())
				}
			};

			ctx.register_animations(&animations, &masks);
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

pub trait AnimationsMarker: Component {
	type TConfig: Asset + AnimationConfig;
	type TConfigComponent: Component + View<Handle<Self::TConfig>>;
}

pub trait AnimationConfig {
	fn animations(&self) -> impl ExactSizeIterator<Item = AnimationKeyAndNames>;
	fn masks(&self) -> impl ExactSizeIterator<Item = AnimationMaskAndBones>;
}

pub type AnimationKeyAndNames = (AnimationKey, Animation<AnimationNames>);
pub type AnimationMaskAndBones = (AnimationMaskBits, AffectedAnimationBones);

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
			key,
			Animation {
				clips: animation.clips.clone().try_map_option(get_clips)?,
				play_mode: animation.play_mode,
				mask_groups: animation.mask_groups,
			},
		))
	}
}

#[derive(Debug, PartialEq, Clone)]
pub enum RegisterAnimationsError<T> {
	MissingAnimations {
		entity: Entity,
		missing: HashSet<AnimationName>,
		available: HashSet<Box<str>>,
	},
	GltfLookupMissing {
		entity: Entity,
	},
	_P((PhantomData<T>, Unreachable)),
}

impl<T> Display for RegisterAnimationsError<T> {
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
			RegisterAnimationsError::GltfLookupMissing { entity } => {
				write!(
					f,
					"{}: missing {}, which is required for {}",
					entity,
					type_name::<GltfLookup>(),
					type_name::<T>()
				)
			}
			RegisterAnimationsError::_P(_) => unreachable!(),
		}
	}
}

impl<T> ErrorData for RegisterAnimationsError<T> {
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
	use crate::{
		bit_mask_index,
		tools::bone_name::BoneName,
		traits::handles_animations::AnimationClips,
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::sync::LazyLock;
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

	#[derive(Component, Debug, PartialEq)]
	struct _Maker;

	impl AnimationsMarker for _Maker {
		type TConfig = _Config;
		type TConfigComponent = _ConfigHandle;
	}

	#[derive(Asset, TypePath, Default)]
	struct _Config {
		animations: Vec<AnimationKeyAndNames>,
		masks: Vec<AnimationMaskAndBones>,
	}

	impl AnimationConfig for _Config {
		fn animations(&self) -> impl ExactSizeIterator<Item = AnimationKeyAndNames> {
			self.animations.iter().cloned()
		}

		fn masks(&self) -> impl ExactSizeIterator<Item = AnimationMaskAndBones> {
			self.masks.iter().cloned()
		}
	}

	#[derive(Component)]
	struct _ConfigHandle(Handle<_Config>);

	impl View<Handle<_Config>> for _ConfigHandle {
		fn view(&self) -> &'_ Handle<_Config> {
			&self.0
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(Result<(), Vec<RegisterAnimationsError<_Maker>>>);

	fn setup<const C: usize, const M: usize>(
		configs: [(&Handle<_Config>, _Config); C],
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
			_Maker::register_animations_system::<Query<&mut _Animations>>.pipe(
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
		let config = _Config {
			animations: Vec::from([(
				AnimationKey::Run,
				Animation {
					clips: AnimationClips::Single(AnimationName::from("Run")),
					..default()
				},
			)]),
			masks: Vec::from([(
				AnimationMaskBits::zero().with_set(bit_mask_index!(2)),
				AffectedAnimationBones {
					from_root: BoneName::from("root"),
					until_exclusive: [].into(),
				},
			)]),
		};
		let config_handle = new_handle();
		let gltf_handle = new_handle();
		let gltf = gltf([("Run", CLIP.clone())]);
		let mut app = setup([(&config_handle, config)], [(&gltf_handle, gltf)]);
		app.world_mut().spawn((
			_Maker,
			GltfLookup(gltf_handle),
			_ConfigHandle(config_handle),
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
		let config = _Config {
			animations: Vec::from([(
				AnimationKey::Run,
				Animation {
					clips: AnimationClips::Single(AnimationName::from("Run")),
					..default()
				},
			)]),
			masks: Vec::from([(
				AnimationMaskBits::zero().with_set(bit_mask_index!(2)),
				AffectedAnimationBones {
					from_root: BoneName::from("root"),
					until_exclusive: [].into(),
				},
			)]),
		};
		let config_handle = new_handle();
		let gltf_handle = new_handle();
		let gltf = gltf([("Run", CLIP.clone())]);
		let mut app = setup([(&config_handle, config)], []);
		app.world_mut().spawn((
			_Maker,
			GltfLookup(gltf_handle.clone()),
			_ConfigHandle(config_handle),
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
		let config = _Config {
			animations: Vec::from([]),
			masks: Vec::from([(
				AnimationMaskBits::zero().with_set(bit_mask_index!(2)),
				AffectedAnimationBones {
					from_root: BoneName::from("root"),
					until_exclusive: [].into(),
				},
			)]),
		};
		let mut app = setup([(&config_handle, config)], []);
		app.world_mut().spawn((
			_Maker,
			_ConfigHandle(config_handle),
			_Animations::new().with_mock(assert_animations_registered),
		));

		app.update();

		fn assert_animations_registered(mock: &mut Mock_Animations) {
			mock.expect_register_animations()
				.once()
				.with(eq(HashMap::from([])), eq(HashMap::from([])))
				.return_const(());
		}
	}

	#[test]
	fn set_empty_animations_when_no_gltf_lookup() {
		let config_handle = new_handle();
		let config = _Config {
			animations: Vec::from([(
				AnimationKey::Run,
				Animation {
					clips: AnimationClips::Single(AnimationName::from("Run")),
					..default()
				},
			)]),
			masks: Vec::from([(
				AnimationMaskBits::zero().with_set(bit_mask_index!(2)),
				AffectedAnimationBones {
					from_root: BoneName::from("root"),
					until_exclusive: [].into(),
				},
			)]),
		};
		let mut app = setup([(&config_handle, config)], []);
		app.world_mut().spawn((
			_Maker,
			_ConfigHandle(config_handle),
			_Animations::new().with_mock(assert_animations_registered),
		));

		app.update();

		fn assert_animations_registered(mock: &mut Mock_Animations) {
			mock.expect_register_animations()
				.once()
				.with(eq(HashMap::from([])), eq(HashMap::from([])))
				.return_const(());
		}
	}

	#[test]
	fn remove_apply_agent_animations_marker() {
		let config_handle = new_handle();
		let gltf_handle = new_handle();
		let mut app = setup(
			[(&config_handle, _Config::default())],
			[(&gltf_handle, gltf([]))],
		);
		let entity = app
			.world_mut()
			.spawn((
				_Maker,
				GltfLookup(gltf_handle),
				_ConfigHandle(config_handle),
				_Animations::default(),
			))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<_Maker>());
	}

	#[test]
	fn do_nothing_if_apply_agent_animation_marker_missing() {
		let config_handle = new_handle();
		let gltf_handle = new_handle();
		let mut app = setup(
			[(&config_handle, _Config::default())],
			[(&gltf_handle, gltf([]))],
		);
		app.world_mut().spawn((
			GltfLookup(gltf_handle),
			_ConfigHandle(config_handle),
			_Animations::new().with_mock(assert_register_not_called),
		));

		app.update();

		fn assert_register_not_called(mock: &mut Mock_Animations) {
			mock.expect_register_animations().never().return_const(());
		}
	}

	#[test]
	fn return_animations_missing() {
		let config = _Config {
			animations: Vec::from([
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
			..default()
		};
		let config_handle = new_handle();
		let gltf_handle = new_handle();
		let gltf = gltf([("Run", new_handle())]);
		let mut app = setup([(&config_handle, config)], [(&gltf_handle, gltf)]);
		let entity = app
			.world_mut()
			.spawn((
				_Maker,
				GltfLookup(gltf_handle),
				_ConfigHandle(config_handle),
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
		let config = _Config {
			animations: Vec::from([(
				AnimationKey::Run,
				Animation {
					clips: AnimationClips::Single(AnimationName::from("Run")),
					..default()
				},
			)]),
			..default()
		};
		let config_handle = new_handle();
		let mut app = setup([(&config_handle, config)], []);
		let entity = app
			.world_mut()
			.spawn((
				_Maker,
				_ConfigHandle(config_handle),
				_Animations::new().with_mock(|mock| {
					mock.expect_register_animations().return_const(());
				}),
			))
			.id();

		app.update();

		assert_eq!(
			&_Result(Err(vec![RegisterAnimationsError::GltfLookupMissing {
				entity
			}])),
			app.world().resource::<_Result>()
		);
	}

	#[test]
	fn return_ok_when_all_animations_found() {
		let config = _Config {
			animations: Vec::from([
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
			_Maker,
			GltfLookup(gltf_handle),
			_ConfigHandle(config_handle),
			_Animations::new().with_mock(|mock| {
				mock.expect_register_animations().return_const(());
			}),
		));

		app.update();

		assert_eq!(&_Result(Ok(())), app.world().resource::<_Result>());
	}

	#[test]
	fn return_ok_when_no_gltf_lookup_and_no_animations() {
		let config_handle = new_handle();
		let mut app = setup([(&config_handle, _Config::default())], []);
		app.world_mut().spawn((
			_Maker,
			_ConfigHandle(config_handle),
			_Animations::new().with_mock(|mock| {
				mock.expect_register_animations().return_const(());
			}),
		));

		app.update();

		assert_eq!(&_Result(Ok(())), app.world().resource::<_Result>());
	}
}
