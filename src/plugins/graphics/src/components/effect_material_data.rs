use crate::components::{camera_labels::CompositePass, model_render_layers::ModelRenderLayers};
use bevy::prelude::*;

#[derive(Component, Debug, PartialEq, Clone)]
#[require(Visibility::Hidden, ModelRenderLayers::from(CompositePass))]
pub struct EffectMaterialData {
	pub(crate) first_pass: Handle<Image>,
	pub(crate) base_color: LinearRgba,
	pub(crate) fresnel_color: LinearRgba,
	pub(crate) flags: u32,
}

impl EffectMaterialData {
	const DEFAULT_COLOR: Srgba = Srgba {
		red: 1.,
		green: 1.,
		blue: 1.,
		alpha: 0.,
	};
	const DEFAULT_FRESNEL: Srgba = Srgba {
		red: 0.,
		green: 0.,
		blue: 0.,
		alpha: 0.,
	};

	pub(crate) fn from_first_pass(first_pass: Handle<Image>) -> Self {
		Self {
			first_pass,
			..default()
		}
	}

	pub(crate) fn add_flag(&mut self, effect: EffectFlag) {
		match effect {
			EffectFlag::BaseColor(base_color) => self.base_color = base_color,
			EffectFlag::Fresnel(fresnel_color) => self.fresnel_color = fresnel_color,
			EffectFlag::Distortion => {}
		}

		self.set_flag_internal(effect, true);
	}

	fn set_flag_internal(&mut self, flag: impl Into<u32>, to: bool) {
		match to {
			true => self.flags |= flag.into(),
			false => self.flags &= !flag.into(),
		}
	}
}

impl Default for EffectMaterialData {
	fn default() -> Self {
		Self {
			first_pass: Handle::default(),
			base_color: LinearRgba::from(Self::DEFAULT_COLOR),
			fresnel_color: LinearRgba::from(Self::DEFAULT_FRESNEL),
			flags: 0,
		}
	}
}

#[derive(Debug, PartialEq)]
pub(crate) enum EffectFlag {
	BaseColor(LinearRgba),
	Fresnel(LinearRgba),
	Distortion,
}

impl EffectFlag {
	pub(crate) fn fresnel(color: impl Into<LinearRgba>) -> Self {
		Self::Fresnel(color.into())
	}

	pub(crate) fn base_color(color: impl Into<LinearRgba>) -> Self {
		Self::BaseColor(color.into())
	}
}

impl From<EffectFlag> for u32 {
	fn from(flag: EffectFlag) -> Self {
		match flag {
			EffectFlag::BaseColor(_) => 1 << 0,
			EffectFlag::Fresnel(_) => 1 << 1,
			EffectFlag::Distortion => 1 << 2,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use test_case::test_case;

	#[test_case(0b0000, 1, 0b0001; "1 to 1")]
	#[test_case(0b0110, 1, 0b0111; "1 added")]
	#[test_case(0b0000, 2, 0b0010; "2 to 2")]
	#[test_case(0b0101, 2, 0b0111; "2 added")]
	fn set_bit(flags: u32, flag: u32, expected: u32) {
		let mut material = EffectMaterialData { flags, ..default() };

		material.set_flag_internal(flag, true);

		assert_eq!(expected, material.flags);
	}

	#[test_case(0b0001, 1, 0b0000; "1 to 0")]
	#[test_case(0b0111, 1, 0b0110; "1 removed")]
	#[test_case(0b0010, 2, 0b0000; "2 to 0")]
	#[test_case(0b0111, 2, 0b0101; "2 removed")]
	fn unset_bit(flags: u32, flag: u32, expected: u32) {
		let mut material = EffectMaterialData { flags, ..default() };

		material.set_flag_internal(flag, false);

		assert_eq!(expected, material.flags);
	}
}
