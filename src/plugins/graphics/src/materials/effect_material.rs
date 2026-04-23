use bevy::{prelude::*, render::render_resource::AsBindGroup, shader::ShaderRef};

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub(crate) struct EffectMaterial {
	#[texture(0)]
	#[sampler(1)]
	first_pass: Handle<Image>,
	#[uniform(2)]
	base_color: LinearRgba,
	#[uniform(3)]
	fresnel_color: LinearRgba,
	#[uniform(4)]
	flags: u32,
}

impl EffectMaterial {
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

	pub(crate) fn add_effect(&mut self, effect: EffectFlag) {
		if let EffectFlag::Fresnel(color) = effect {
			self.fresnel_color = color;
		}

		self.set_flag_internal(effect, true);
	}

	pub(crate) fn remove_effect(&mut self, effect: EffectFlag) {
		if let EffectFlag::Fresnel(_) = effect {
			self.fresnel_color = Self::DEFAULT_COLOR.into();
		}

		self.set_flag_internal(effect, false);
	}

	fn set_flag_internal(&mut self, flag: impl Into<u32>, to: bool) {
		match to {
			true => self.flags |= flag.into(),
			false => self.flags &= !flag.into(),
		}
	}
}

impl Default for EffectMaterial {
	fn default() -> Self {
		Self {
			first_pass: Handle::default(),
			base_color: Self::DEFAULT_COLOR.into(),
			fresnel_color: Self::DEFAULT_FRESNEL.into(),
			flags: 0,
		}
	}
}

impl From<FirstPassImage> for EffectMaterial {
	fn from(FirstPassImage(first_pass): FirstPassImage) -> Self {
		Self {
			first_pass,
			..default()
		}
	}
}

impl Material for EffectMaterial {
	fn fragment_shader() -> ShaderRef {
		"shaders/effect_shader.wgsl".into()
	}

	fn alpha_mode(&self) -> AlphaMode {
		AlphaMode::Blend
	}

	fn enable_shadows() -> bool {
		false
	}
}

#[derive(Debug, PartialEq)]
pub(crate) struct FirstPassImage(pub(crate) Handle<Image>);

#[derive(Debug, PartialEq)]
pub(crate) enum EffectFlag {
	BaseColor(LinearRgba),
	Fresnel(LinearRgba),
	Distortion,
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
		let mut material = EffectMaterial { flags, ..default() };

		material.set_flag_internal(flag, true);

		assert_eq!(expected, material.flags);
	}

	#[test_case(0b0001, 1, 0b0000; "1 to 0")]
	#[test_case(0b0111, 1, 0b0110; "1 removed")]
	#[test_case(0b0010, 2, 0b0000; "2 to 0")]
	#[test_case(0b0111, 2, 0b0101; "2 removed")]
	fn unset_bit(flags: u32, flag: u32, expected: u32) {
		let mut material = EffectMaterial { flags, ..default() };

		material.set_flag_internal(flag, false);

		assert_eq!(expected, material.flags);
	}
}
