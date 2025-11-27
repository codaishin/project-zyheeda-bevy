mod priority_order;

use crate::{
	errors::{ErrorData, Level},
	tools::{action_key::slot::SlotKey, path::Path},
	traits::{
		accessors::get::GetContextMut,
		handles_animations::priority_order::DescendingAnimationPriorities,
	},
};
use bevy::{ecs::system::SystemParam, prelude::*};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};
use std::{
	collections::{HashMap, HashSet},
	fmt::Display,
	ops::{Deref, DerefMut},
	sync::Arc,
};

pub trait HandlesAnimations {
	type TAnimationsMut<'w, 's>: SystemParam
		+ for<'c> GetContextMut<Animations, TContext<'c>: RegisterAnimations2>
		+ for<'c> GetContextMut<Animations, TContext<'c>: ActiveAnimationsMut>
		+ for<'c> GetContextMut<Animations, TContext<'c>: MoveDirectionMut>;
}

pub struct Animations {
	pub entity: Entity,
}

impl From<Animations> for Entity {
	fn from(Animations { entity }: Animations) -> Self {
		entity
	}
}

pub type AnimationsSystemParamMut<'w, 's, T> = <T as HandlesAnimations>::TAnimationsMut<'w, 's>;

pub trait RegisterAnimations2 {
	fn register_animations(
		&mut self,
		animations: &HashMap<AnimationKey, Animation2>,
		animation_mask_groups: &HashMap<AnimationMaskBits, AffectedAnimationBones2>,
	);
}

impl<T> RegisterAnimations2 for T
where
	T: DerefMut<Target: RegisterAnimations2>,
{
	fn register_animations(
		&mut self,
		animations: &HashMap<AnimationKey, Animation2>,
		animation_mask_groups: &HashMap<AnimationMaskBits, AffectedAnimationBones2>,
	) {
		self.deref_mut()
			.register_animations(animations, animation_mask_groups);
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

impl<T> ActiveAnimations for T
where
	T: Deref<Target: ActiveAnimations>,
{
	fn active_animations<TLayer>(
		&self,
		layer: TLayer,
	) -> Result<&HashSet<AnimationKey>, AnimationsUnprepared>
	where
		TLayer: Into<AnimationPriority>,
	{
		self.deref().active_animations(layer)
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

impl<T> ActiveAnimationsMut for T
where
	T: DerefMut<Target: ActiveAnimationsMut>,
{
	fn active_animations_mut<TLayer>(
		&mut self,
		layer: TLayer,
	) -> Result<&mut HashSet<AnimationKey>, AnimationsUnprepared>
	where
		TLayer: Into<AnimationPriority>,
	{
		self.deref_mut().active_animations_mut(layer)
	}
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

pub trait MoveDirection {
	fn move_direction(&self) -> Option<Dir3>;
}

impl<T> MoveDirection for T
where
	T: Deref<Target: MoveDirection>,
{
	fn move_direction(&self) -> Option<Dir3> {
		self.deref().move_direction()
	}
}

pub trait MoveDirectionMut: MoveDirection {
	fn move_direction_mut(&mut self) -> &mut Option<Dir3>;
}

impl<T> MoveDirectionMut for T
where
	T: DerefMut<Target: MoveDirectionMut>,
{
	fn move_direction_mut(&mut self) -> &mut Option<Dir3> {
		self.deref_mut().move_direction_mut()
	}
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
	pub mask_groups: AnimationMaskBits,
}

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone, Copy, Serialize, Deserialize)]
pub struct AnimationMaskBits(#[serde(with = "bits_conversion")] AnimationMask);

impl AnimationMaskBits {
	pub const ZERO: Self = Self::zero();

	pub const fn zero() -> Self {
		Self(0)
	}

	pub fn to_animation_mask(&self) -> AnimationMask {
		self.0
	}

	pub fn with_set(mut self, bit: BitMaskIndex) -> Self {
		self.set(bit);
		self
	}

	pub fn set(&mut self, BitMaskIndex(bit): BitMaskIndex) {
		self.0 |= 1 << bit;
	}
}

pub struct BitMaskIndex(u8);

#[macro_export]
macro_rules! bit_mask_index {
	($bit:expr) => {{
		type BitMaskIndex = $crate::traits::handles_animations::BitMaskIndex;
		const INDEX: BitMaskIndex = match BitMaskIndex::try_parse($bit) {
			Ok(index) => index,
			Err(_) => panic!("invalid BitMaskIndex"),
		};
		INDEX
	}};
}

impl BitMaskIndex {
	const MAX_BIT_INDEX: u8 = 63;

	pub const MAX_INDEX: Self = bit_mask_index!(BitMaskIndex::MAX_BIT_INDEX);

	pub const fn try_parse(index: u8) -> Result<Self, MaxBitExceeded> {
		if index > Self::MAX_BIT_INDEX {
			return Err(MaxBitExceeded);
		}

		Ok(Self(index))
	}
}

impl TryFrom<u8> for BitMaskIndex {
	type Error = MaxBitExceeded;

	fn try_from(bit: u8) -> Result<Self, Self::Error> {
		Self::try_parse(bit)
	}
}

#[derive(Debug)]
pub struct MaxBitExceeded;

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

mod bits_conversion {
	use super::*;

	pub(super) const DESERIALIZE_ERROR_PREFIX: &str = "Failed to parse animation mask";

	#[derive(Deserialize)]
	#[serde(untagged)]
	enum U64OrString {
		U64(u64),
		String(String),
	}

	pub(crate) fn deserialize<'a, D>(deserializer: D) -> Result<AnimationMask, D::Error>
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

	pub(crate) fn serialize<S>(mask: &AnimationMask, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		match mask {
			0 => "".serialize(serializer),
			mask => format!("{mask:b}").serialize(serializer),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	struct _Wrapper {
		#[serde(with = "bits_conversion")]
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
		fn parse_error_invalid_digits() {
			let value = json! ({
				"mask": "123"
			});

			let Err(error) = serde_json::from_value::<_Wrapper>(value) else {
				panic!("EXPECTED ERROR, BUT WAS VALUE");
			};

			assert_eq!(
				format!(
					"{}: invalid digit found in string",
					bits_conversion::DESERIALIZE_ERROR_PREFIX
				),
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
