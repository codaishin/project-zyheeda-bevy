mod priority_order;

use super::iteration::IterFinite;
use crate::{
	errors::{ErrorData, Level},
	tools::{action_key::slot::SlotKey, path::Path},
	traits::{
		accessors::get::GetContextMut,
		animation::priority_order::DescendingAnimationPriorities,
	},
};
use bevy::{
	ecs::{component::Mutable, system::SystemParam},
	prelude::*,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};
use std::{
	collections::{HashMap, HashSet},
	fmt::Display,
	ops::DerefMut,
	sync::Arc,
};

pub trait HandlesAnimations {
	type TAnimationsMut<'w, 's>: SystemParam
		+ for<'c> GetContextMut<Animations, TContext<'c>: RegisterAnimations2>
		+ for<'c> GetContextMut<Animations, TContext<'c>: ActiveAnimationsMut>
		+ for<'c> GetContextMut<Animations, TContext<'c>: SetMovementDirection>;
}

pub struct Animations {
	pub entity: Entity,
}

impl From<Animations> for Entity {
	fn from(Animations { entity }: Animations) -> Self {
		entity
	}
}

pub type AnimationsParamMut<'w, 's, T> = <T as HandlesAnimations>::TAnimationsMut<'w, 's>;

pub trait RegisterAnimations2 {
	fn register_animations(&mut self, animations: &HashMap<AnimationKey, Animation2>);
}

impl<T> RegisterAnimations2 for T
where
	T: DerefMut<Target: RegisterAnimations2>,
{
	fn register_animations(&mut self, animations: &HashMap<AnimationKey, Animation2>) {
		self.deref_mut().register_animations(animations);
	}
}

pub trait ActiveAnimations {
	fn active_animations<TLayer>(
		&self,
		layer: TLayer,
	) -> Result<&HashSet<AnimationKey>, AnimationsUnprepared>
	where
		TLayer: Into<AnimationPriority>;
}

#[derive(Debug, PartialEq)]
pub struct AnimationsUnprepared {
	pub entity: Entity,
}

impl ErrorData for AnimationsUnprepared {
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl Display {
		"Animations unprepared"
	}

	fn into_details(self) -> impl Display {
		format!(
			"Tried to retrieve animations for {:?}, but animations have not been registered (yet)",
			self.entity
		)
	}
}

pub trait ActiveAnimationsMut: ActiveAnimations {
	fn active_animations_mut<TLayer>(
		&mut self,
		layer: TLayer,
	) -> Result<&mut HashSet<AnimationKey>, AnimationsUnprepared>
	where
		TLayer: Into<AnimationPriority>;
}

pub trait SetMovementDirection {
	fn set_movement_direction(&mut self, direction: Dir3);
}

pub trait StartAnimation {
	fn start_animation<TLayer>(&mut self, layer: TLayer, animation: Animation)
	where
		TLayer: Into<AnimationPriority> + 'static;
}

pub trait SetAnimations {
	fn set_animations<TLayer, TAnimations>(&mut self, layer: TLayer, animations: TAnimations)
	where
		TLayer: Into<AnimationPriority> + 'static,
		TAnimations: IntoIterator<Item = Animation> + 'static;
}

pub trait StopAnimation {
	fn stop_animation<TLayer>(&mut self, layer: TLayer)
	where
		TLayer: Into<AnimationPriority> + 'static;
}

pub trait GetAnimationDefinitions
where
	for<'a> AnimationMask: From<&'a Self::TAnimationMask>,
	for<'a> AffectedAnimationBones: From<&'a Self::TAnimationMask>,
{
	type TAnimationMask: IterFinite;

	fn animations() -> HashMap<AnimationPath, AnimationMask>;
}

#[derive(Debug, PartialEq, Clone)]
pub enum AffectedAnimationBones {
	SubTree {
		root: Name,
		until_exclusive: Vec<Name>,
	},
	Leaf {
		root: Name,
	},
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum AnimationPath {
	Single(Path),
	Directional(Directional),
}

impl From<&'static str> for AnimationPath {
	fn from(path: &'static str) -> Self {
		Self::Single(Path::from(path))
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct Directional {
	pub forward: Path,
	pub backward: Path,
	pub left: Path,
	pub right: Path,
}

// FIXME: remove when consumers moved to new `HandlesAnimations` interface
pub trait HasAnimationsDispatch {
	type TAnimationDispatch: StartAnimation
		+ SetAnimations
		+ StopAnimation
		+ Component<Mutability = Mutable>;
}

pub trait ConfigureNewAnimationDispatch {
	fn configure_animation_dispatch(
		&self,
		new_animation_dispatch: &mut (impl StartAnimation + StopAnimation),
	);
}

pub trait GetMovementDirection {
	fn movement_direction(&self, transform: &GlobalTransform) -> Option<Dir3>;
}

pub trait RegisterAnimations: HasAnimationsDispatch {
	fn register_animations<TAgent>(app: &mut App)
	where
		TAgent: Component + GetAnimationDefinitions + ConfigureNewAnimationDispatch,
		for<'a> AnimationMask: From<&'a TAgent::TAnimationMask>,
		for<'a> AffectedAnimationBones: From<&'a TAgent::TAnimationMask>;

	fn register_movement_direction<TMovementDirection>(app: &mut App)
	where
		TMovementDirection: Component + GetMovementDirection;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum AnimationPriority {
	High,
	Medium,
	Low,
}

impl AnimationPriority {
	pub fn ordered_descending() -> DescendingAnimationPriorities {
		DescendingAnimationPriorities::default()
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone, Copy, Serialize, Deserialize)]
pub enum PlayMode {
	#[default]
	Replay,
	Repeat,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct Animation {
	pub path: AnimationPath,
	pub play_mode: PlayMode,
}

impl Animation {
	pub const fn new(path: AnimationPath, play_mode: PlayMode) -> Self {
		Self { path, play_mode }
	}
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Animation2 {
	pub path: AnimationPath,
	pub play_mode: PlayMode,
	#[serde(deserialize_with = "bits_to_mask", serialize_with = "mask_to_bits")]
	pub mask: AnimationMask,
	pub bones: AffectedAnimationBones2,
}

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub struct AffectedAnimationBones2 {
	pub from_root: BoneName,
	#[serde(default)]
	pub until_exclusive: HashSet<BoneName>,
}

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone, Serialize, Deserialize)]
pub struct BoneName(Arc<str>);

impl From<&str> for BoneName {
	fn from(value: &str) -> Self {
		Self(Arc::from(value))
	}
}

impl From<&Name> for BoneName {
	fn from(value: &Name) -> Self {
		Self(Arc::from(value.as_str()))
	}
}

impl PartialEq<Name> for BoneName {
	fn eq(&self, other: &Name) -> bool {
		&*self.0 == other.as_str()
	}
}

impl PartialEq<BoneName> for Name {
	fn eq(&self, other: &BoneName) -> bool {
		self.as_str() == &*other.0
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum AnimationKey {
	Idle,
	Walk,
	Run,
	Skill(SlotKey),
}

const DESERIALIZE_ERROR_PREFIX: &str = "Failed to parse animation mask";

#[derive(Deserialize)]
#[serde(untagged)]
enum U64OrString {
	U64(u64),
	String(String),
}

pub(crate) fn bits_to_mask<'a, D>(deserializer: D) -> Result<AnimationMask, D::Error>
where
	D: Deserializer<'a>,
{
	match U64OrString::deserialize(deserializer)? {
		U64OrString::U64(mask) => Ok(mask),
		U64OrString::String(bits) if bits.is_empty() => Ok(0),
		U64OrString::String(bits) => AnimationMask::from_str_radix(&bits, 2)
			.map_err(|error| Error::custom(format!("{DESERIALIZE_ERROR_PREFIX}: {error}"))),
	}
}

pub(crate) fn mask_to_bits<S>(mask: &AnimationMask, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	match mask {
		0 => "".serialize(serializer),
		mask => format!("{mask:b}").serialize(serializer),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	struct _Wrapper {
		#[serde(deserialize_with = "bits_to_mask", serialize_with = "mask_to_bits")]
		mask: AnimationMask,
	}

	mod bits_to_mask {
		use super::*;
		use test_case::test_case;

		#[test_case("0"; "0")]
		#[test_case(""; "empty")]
		fn deserialize_zero(v: &str) {
			let value = json! ({
				"mask": v
			});

			let mask = serde_json::from_value::<_Wrapper>(value).expect("DESERIALIZE FAILED");

			assert_eq!(_Wrapper { mask: 0 }, mask);
		}

		#[test_case("101", 5; "5")]
		#[test_case("111", 7; "7")]
		#[test_case("10111", 23; "23")]
		fn deserialize_bits(v: &str, mask: AnimationMask) {
			let value = json! ({
				"mask": v
			});

			let wrapper = serde_json::from_value::<_Wrapper>(value).expect("DESERIALIZE FAILED");

			assert_eq!(_Wrapper { mask }, wrapper);
		}

		#[test_case(5; "5")]
		#[test_case(7; "7")]
		fn deserialize_raw_number(mask: AnimationMask) {
			let value = json! ({
				"mask": mask
			});

			let wrapper = serde_json::from_value::<_Wrapper>(value).expect("DESERIALIZE FAILED");

			assert_eq!(_Wrapper { mask }, wrapper);
		}

		#[test]
		fn parse_error() {
			let value = json! ({
				"mask": "123"
			});

			let Err(error) = serde_json::from_value::<_Wrapper>(value) else {
				panic!("EXPECTED ERROR");
			};

			assert_eq!(
				format!("{DESERIALIZE_ERROR_PREFIX}: invalid digit found in string"),
				error.to_string(),
			)
		}
	}

	mod mask_to_bits {
		use super::*;
		use test_case::test_case;

		#[test]
		fn serialize_zero() {
			let wrapper = _Wrapper { mask: 0 };

			let value = serde_json::to_value(wrapper).expect("SERIALIZE FAILED");

			assert_eq!(
				json! ({
					"mask": ""
				}),
				value
			);
		}

		#[test_case(5,"101"; "5")]
		#[test_case(7,"111"; "7")]
		#[test_case(23,"10111"; "23")]
		fn serialize_bits(mask: AnimationMask, v: &str) {
			let wrapper = _Wrapper { mask };

			let value = serde_json::to_value(wrapper).expect("SERIALIZE FAILED");

			assert_eq!(
				json! ({
					"mask": v
				}),
				value
			);
		}
	}
}
