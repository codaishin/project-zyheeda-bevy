mod priority_order;

use crate::{
	tools::{action_key::slot::SlotKey, bone_name::BoneName},
	traits::{
		accessors::get::GetContextMut,
		handles_animations::priority_order::DescendingAnimationPriorities,
	},
};
use bevy::{ecs::system::SystemParam, prelude::*};
use macros::{EntityKey, InRange};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};
use std::{
	borrow::{Borrow, Cow},
	collections::{HashMap, HashSet},
	hash::Hash,
	ops::{Deref, DerefMut},
};
use zyheeda_core::prelude::*;

pub trait HandlesAnimations {
	type TAnimationsMut: SystemParam
		+ for<'c> GetContextMut<WithoutAnimations, TContext<'c>: RegisterAnimations>
		+ for<'c> GetContextMut<Animations, TContext<'c>: ActiveAnimationsMut>
		+ for<'c> GetContextMut<Animations, TContext<'c>: GetMoveDirectionMut>
		+ for<'c> GetContextMut<Animations, TContext<'c>: GetForwardPitchMut>;
}

#[derive(EntityKey)]
pub struct WithoutAnimations {
	pub entity: Entity,
}

#[derive(EntityKey)]
pub struct Animations {
	pub entity: Entity,
}

pub trait RegisterAnimations {
	fn register_animations(
		&mut self,
		animations: &HashMap<AnimationKey, Animation>,
		animation_mask_groups: &HashMap<AnimationMaskBits, AffectedAnimationBones>,
	);
}

impl<T> RegisterAnimations for T
where
	T: DerefMut<Target: RegisterAnimations>,
{
	fn register_animations(
		&mut self,
		animations: &HashMap<AnimationKey, Animation>,
		animation_mask_groups: &HashMap<AnimationMaskBits, AffectedAnimationBones>,
	) {
		self.deref_mut()
			.register_animations(animations, animation_mask_groups);
	}
}

pub trait ActiveAnimations {
	fn active_animations<TLayer>(&self, layer: TLayer) -> &OrderedSet<AnimationKey>
	where
		TLayer: Into<AnimationPriority>;
}

impl<T> ActiveAnimations for T
where
	T: Deref<Target: ActiveAnimations>,
{
	fn active_animations<TLayer>(&self, layer: TLayer) -> &OrderedSet<AnimationKey>
	where
		TLayer: Into<AnimationPriority>,
	{
		self.deref().active_animations(layer)
	}
}

pub trait ActiveAnimationsMut: ActiveAnimations {
	fn active_animations_mut<TLayer>(&mut self, layer: TLayer) -> &mut OrderedSet<AnimationKey>
	where
		TLayer: Into<AnimationPriority>;
}

impl<T> ActiveAnimationsMut for T
where
	T: DerefMut<Target: ActiveAnimationsMut>,
{
	fn active_animations_mut<TLayer>(&mut self, layer: TLayer) -> &mut OrderedSet<AnimationKey>
	where
		TLayer: Into<AnimationPriority>,
	{
		self.deref_mut().active_animations_mut(layer)
	}
}

pub trait GetMoveDirection {
	fn get_move_direction(&self) -> Option<Dir3>;
}

impl<T> GetMoveDirection for T
where
	T: Deref<Target: GetMoveDirection>,
{
	fn get_move_direction(&self) -> Option<Dir3> {
		self.deref().get_move_direction()
	}
}

pub trait GetMoveDirectionMut: GetMoveDirection {
	fn get_move_direction_mut(&mut self) -> &mut Option<Dir3>;
}

impl<T> GetMoveDirectionMut for T
where
	T: DerefMut<Target: GetMoveDirectionMut>,
{
	fn get_move_direction_mut(&mut self) -> &mut Option<Dir3> {
		self.deref_mut().get_move_direction_mut()
	}
}

pub trait GetForwardPitch {
	fn get_forward_pitch(&self) -> Option<DirForwardPitch>;
}

impl<T> GetForwardPitch for T
where
	T: Deref<Target: GetForwardPitch>,
{
	fn get_forward_pitch(&self) -> Option<DirForwardPitch> {
		self.deref().get_forward_pitch()
	}
}

pub trait GetForwardPitchMut: GetForwardPitch {
	fn get_forward_pitch_mut(&mut self) -> &mut Option<DirForwardPitch>;
}

impl<T> GetForwardPitchMut for T
where
	T: DerefMut<Target: GetForwardPitchMut>,
{
	fn get_forward_pitch_mut(&mut self) -> &mut Option<DirForwardPitch> {
		self.deref_mut().get_forward_pitch_mut()
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum AnimationClips<T = Handle<AnimationClip>> {
	Single(T),
	Directional(Directional<T>),
	PitchedForward(PitchedForward<T>),
}

impl<T> AnimationClips<T> {
	pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> AnimationClips<U> {
		match self {
			AnimationClips::Single(clip) => AnimationClips::Single(f(clip)),
			AnimationClips::Directional(Directional {
				forward,
				backward,
				left,
				right,
			}) => AnimationClips::Directional(Directional {
				forward: f(forward),
				backward: f(backward),
				left: f(left),
				right: f(right),
			}),
			AnimationClips::PitchedForward(PitchedForward { neutral, up, down }) => {
				AnimationClips::PitchedForward(PitchedForward {
					neutral: f(neutral),
					up: (up.0, f(up.1)),
					down: (down.0, f(down.1)),
				})
			}
		}
	}

	pub fn try_map_option<U>(self, mut f: impl FnMut(T) -> Option<U>) -> Option<AnimationClips<U>> {
		let clips = match self {
			AnimationClips::Single(clip) => AnimationClips::Single(f(clip)?),
			AnimationClips::Directional(Directional {
				forward,
				backward,
				left,
				right,
			}) => AnimationClips::Directional(Directional {
				forward: f(forward)?,
				backward: f(backward)?,
				left: f(left)?,
				right: f(right)?,
			}),
			AnimationClips::PitchedForward(PitchedForward { neutral, up, down }) => {
				AnimationClips::PitchedForward(PitchedForward {
					neutral: f(neutral)?,
					up: (up.0, f(up.1)?),
					down: (down.0, f(down.1)?),
				})
			}
		};

		Some(clips)
	}

	pub fn try_map_result<U, E>(
		self,
		mut f: impl FnMut(T) -> Result<U, E>,
	) -> Result<AnimationClips<U>, E> {
		let clips = match self {
			AnimationClips::Single(clip) => AnimationClips::Single(f(clip)?),
			AnimationClips::Directional(Directional {
				forward,
				backward,
				left,
				right,
			}) => AnimationClips::Directional(Directional {
				forward: f(forward)?,
				backward: f(backward)?,
				left: f(left)?,
				right: f(right)?,
			}),
			AnimationClips::PitchedForward(PitchedForward { neutral, up, down }) => {
				AnimationClips::PitchedForward(PitchedForward {
					neutral: f(neutral)?,
					up: (up.0, f(up.1)?),
					down: (down.0, f(down.1)?),
				})
			}
		};

		Ok(clips)
	}
}

impl<T> Default for AnimationClips<T>
where
	T: Default,
{
	fn default() -> Self {
		Self::Single(T::default())
	}
}

impl<T> From<&'static str> for AnimationClips<T>
where
	T: From<&'static str>,
{
	fn from(path: &'static str) -> Self {
		Self::Single(T::from(path))
	}
}

impl<'a, T> Iterate<'a> for AnimationClips<T>
where
	Self: 'a,
{
	type TItem = &'a T;
	type TIter = Iter<'a, T>;

	fn iterate(&'a self) -> Self::TIter {
		Iter {
			clips: self,
			index: 0,
		}
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct Directional<T = Handle<AnimationClip>> {
	pub forward: T,
	pub backward: T,
	pub left: T,
	pub right: T,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct PitchedForward<T = Handle<AnimationClip>> {
	pub neutral: T,
	pub up: (ForwardPitch, T),
	pub down: (ForwardPitch, T),
}

#[derive(InRange, Debug, PartialEq, Clone, Copy, Serialize)]
#[in_range(low = >0., high = 1.)]
pub struct ForwardPitch(f32);

#[macro_export]
macro_rules! forward_pitch {
	($v:literal) => {{
		const P: $crate::traits::handles_animations::ForwardPitch =
			match $crate::traits::handles_animations::ForwardPitch::try_new($v) {
				Ok(v) => v,
				Err(_) => panic!("value out of range"),
			};

		P
	}};
}

impl ForwardPitch {
	pub const MAX: Self = Self(1.);
}

impl Default for ForwardPitch {
	fn default() -> Self {
		Self::MAX
	}
}

impl Eq for ForwardPitch {}

impl Hash for ForwardPitch {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		let bits = match self.0 {
			0.0 => 0,
			v => v.to_bits(),
		};

		bits.hash(state);
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DirForwardPitch {
	Up(ForwardPitch),
	Down(ForwardPitch),
}

pub struct Iter<'a, T> {
	clips: &'a AnimationClips<T>,
	index: usize,
}

impl<'a, T> Iterator for Iter<'a, T> {
	type Item = &'a T;

	fn next(&mut self) -> Option<Self::Item> {
		let i = self.index;

		self.index += 1;

		match (i, self.clips) {
			(0, AnimationClips::Single(clip)) => Some(clip),
			(0, AnimationClips::Directional(Directional { forward, .. })) => Some(forward),
			(1, AnimationClips::Directional(Directional { backward, .. })) => Some(backward),
			(2, AnimationClips::Directional(Directional { left, .. })) => Some(left),
			(3, AnimationClips::Directional(Directional { right, .. })) => Some(right),
			(0, AnimationClips::PitchedForward(PitchedForward { neutral, .. })) => Some(neutral),
			(1, AnimationClips::PitchedForward(PitchedForward { up, .. })) => Some(&up.1),
			(2, AnimationClips::PitchedForward(PitchedForward { down, .. })) => Some(&down.1),
			_ => None,
		}
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone, Serialize, Deserialize)]
pub struct AnimationName(Cow<'static, str>);

impl AnimationName {
	pub const fn owned(name: String) -> Self {
		Self(Cow::Owned(name))
	}

	pub const fn borrowed(name: &'static str) -> Self {
		Self(Cow::Borrowed(name))
	}
}

impl<T> From<T> for AnimationName
where
	T: Into<Cow<'static, str>>,
{
	fn from(name: T) -> Self {
		Self(name.into())
	}
}

impl Deref for AnimationName {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.0.deref()
	}
}

impl Borrow<str> for AnimationName {
	fn borrow(&self) -> &str {
		self.0.borrow()
	}
}

pub type AnimationNames = AnimationClips<AnimationName>;

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
	Once,
	Replay,
	Repeat,
}

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub struct Animation<TClips = AnimationClips<Handle<AnimationClip>>> {
	pub clips: TClips,
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
pub struct AffectedAnimationBones {
	pub from_root: BoneName,
	#[serde(default)]
	pub until_exclusive: HashSet<BoneName>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum AnimationKey {
	Idle,
	Walk,
	Run,
	Skill {
		slot: SlotKey,
		animation: SkillAnimation,
	},
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum SkillAnimation {
	Aim,
	Block,
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
	#![allow(clippy::expect_used)]
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

	mod animation_clips {
		use super::*;

		#[test]
		fn iter_single_clip() {
			let clips = AnimationClips::Single("single");

			let clips = clips.iterate().take(10).copied().collect::<Vec<_>>();

			assert_eq!(vec!["single"], clips)
		}

		#[test]
		fn iter_directional_clips() {
			let clips = AnimationClips::Directional(Directional {
				forward: "f",
				backward: "b",
				left: "l",
				right: "r",
			});

			let clips = clips.iterate().take(10).copied().collect::<Vec<_>>();

			assert_eq!(vec!["f", "b", "l", "r"], clips)
		}

		#[test]
		fn iter_pitched_clips() {
			let clips = AnimationClips::PitchedForward(PitchedForward {
				neutral: "n",
				up: (ForwardPitch::MAX, "u"),
				down: (ForwardPitch::MAX, "d"),
			});

			let clips = clips.iterate().take(10).copied().collect::<Vec<_>>();

			assert_eq!(vec!["n", "u", "d"], clips)
		}
	}
}
