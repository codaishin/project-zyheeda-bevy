use crate::{
	components::effect_material_data::EffectMaterialData,
	systems::propagate_material::UpdateMaterial,
};
use bevy::{
	prelude::*,
	render::{
		render_resource::{AsBindGroup, ShaderType},
		storage::ShaderBuffer,
	},
	shader::ShaderRef,
};
use common::prelude::*;
use std::sync::LazyLock;

pub static NO_IMPACTS: LazyLock<Handle<ShaderBuffer>> = LazyLock::new(Handle::default);

#[derive(Asset, TypePath, AsBindGroup, Debug, PartialEq, Clone)]
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
	#[storage(5, read_only)]
	impacts: Handle<ShaderBuffer>,
}

impl EffectMaterial {
	fn compute_impacts(impacts: &[Impact]) -> ShaderBuffer {
		let to_local = |Impact { position, strength }: &Impact| {
			let position = Vec3::from(position.0);
			let strength = **strength;

			LocalImpact { position, strength }
		};

		ShaderBuffer::from(impacts.iter().map(to_local).collect::<Vec<_>>())
	}

	fn empty_impacts(&mut self) {
		self.impacts = NO_IMPACTS.clone();
	}

	fn new_impacts(&mut self, buffers: &mut Assets<ShaderBuffer>, impacts: &[Impact]) {
		self.impacts = buffers.add(Self::compute_impacts(impacts));
	}

	fn updated_impacts(&mut self, buffers: &mut Assets<ShaderBuffer>, impacts: &[Impact]) {
		if let Some(mut buffer) = buffers.get_mut(&self.impacts) {
			*buffer = Self::compute_impacts(impacts);
		} else {
			self.impacts = buffers.add(Self::compute_impacts(impacts));
		}
	}
}

impl Default for EffectMaterial {
	fn default() -> Self {
		Self {
			first_pass: default(),
			base_color: default(),
			fresnel_color: default(),
			flags: default(),
			impacts: NO_IMPACTS.clone(),
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

#[derive(Debug, PartialEq, ShaderType)]
pub(crate) struct LocalImpact {
	position: Vec3,
	strength: f32,
}

impl UpdateMaterial for EffectMaterial {
	type TData = EffectMaterialData;
	type TBuffer = Assets<ShaderBuffer>;

	fn update_material(&mut self, buffers: &mut Assets<ShaderBuffer>, data: &EffectMaterialData) {
		let EffectMaterialData {
			first_pass,
			base_color,
			fresnel_color,
			flags,
			impacts,
		} = data;

		self.base_color = *base_color;
		self.fresnel_color = *fresnel_color;
		self.first_pass = first_pass.clone();
		self.flags = *flags;

		let impacts = impacts.as_slice();

		match impacts {
			[] => self.empty_impacts(),
			_ if self.impacts == *NO_IMPACTS => self.new_impacts(buffers, impacts),
			_ => self.updated_impacts(buffers, impacts),
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use bevy::color::palettes::css::{RED, WHITE};
	use testing::new_handle;
	use zyheeda_core::prelude::*;

	#[test]
	fn set_data() {
		let mut buffers = Assets::default();
		let mut material = EffectMaterial::default();
		let first_pass = new_handle();

		material.update_material(
			&mut buffers,
			&EffectMaterialData {
				first_pass: first_pass.clone(),
				base_color: LinearRgba::from(WHITE),
				fresnel_color: LinearRgba::from(RED),
				flags: 2,
				..default()
			},
		);

		assert_eq!(
			EffectMaterial {
				first_pass,
				base_color: LinearRgba::from(WHITE),
				fresnel_color: LinearRgba::from(RED),
				flags: 2,
				..default()
			},
			material
		);
	}

	#[test]
	fn set_impacts() {
		let mut buffers = Assets::default();
		let mut material = EffectMaterial::default();

		material.update_material(
			&mut buffers,
			&EffectMaterialData {
				impacts: vec![Impact {
					position: GlobalVec3(vec_not_nan!(1., 2., 3.)),
					strength: new_f32!(ImpactStrength(0.5)),
				}],
				..default()
			},
		);

		assert_eq!(
			Some(
				&ShaderBuffer::from(vec![LocalImpact {
					position: Vec3::new(1., 2., 3.),
					strength: 0.5
				}])
				.data
			),
			buffers.get(&material.impacts).map(|b| &b.data)
		);
	}

	#[test]
	fn do_not_use_default_handle() {
		let mut buffers = Assets::default();
		let mut material = EffectMaterial::default();

		material.update_material(
			&mut buffers,
			&EffectMaterialData {
				impacts: vec![Impact {
					position: GlobalVec3(vec_not_nan!(1., 2., 3.)),
					strength: new_f32!(ImpactStrength(0.5)),
				}],
				..default()
			},
		);

		assert_ne!(*NO_IMPACTS, material.impacts);
	}

	#[test]
	fn use_default_handle_when_impacts_empty() {
		let mut buffers = Assets::default();
		let mut material = EffectMaterial::default();

		material.update_material(
			&mut buffers,
			&EffectMaterialData {
				impacts: vec![],
				..default()
			},
		);

		assert_eq!(*NO_IMPACTS, material.impacts);
	}

	#[test]
	fn use_default_handle_when_impacts_empty_after_some_impacts_were_set() {
		let mut buffers = Assets::default();
		let mut material = EffectMaterial::default();

		material.update_material(
			&mut buffers,
			&EffectMaterialData {
				impacts: vec![Impact {
					position: GlobalVec3(VecNotNan::default()),
					strength: new_f32!(ImpactStrength(1.0)),
				}],
				..default()
			},
		);
		material.update_material(
			&mut buffers,
			&EffectMaterialData {
				impacts: vec![],
				..default()
			},
		);

		assert_eq!(*NO_IMPACTS, material.impacts);
	}

	#[test]
	fn reuse_handle() {
		let mut buffers = Assets::default();
		let mut material = EffectMaterial::default();
		material.update_material(
			&mut buffers,
			&EffectMaterialData {
				impacts: vec![Impact {
					position: GlobalVec3(vec_not_nan!(1., 2., 3.)),
					strength: new_f32!(ImpactStrength(0.5)),
				}],
				..default()
			},
		);

		let first_handle = material.impacts.clone();
		material.update_material(
			&mut buffers,
			&EffectMaterialData {
				impacts: vec![Impact {
					position: GlobalVec3(vec_not_nan!(1., 2., 3.)),
					strength: new_f32!(ImpactStrength(0.4)),
				}],
				..default()
			},
		);

		assert_eq!(first_handle, material.impacts);
	}

	#[test]
	fn set_new_impacts() {
		let mut buffers = Assets::default();
		let mut material = EffectMaterial::default();
		material.update_material(
			&mut buffers,
			&EffectMaterialData {
				impacts: vec![Impact {
					position: GlobalVec3(vec_not_nan!(1., 2., 3.)),
					strength: new_f32!(ImpactStrength(0.5)),
				}],
				..default()
			},
		);

		material.update_material(
			&mut buffers,
			&EffectMaterialData {
				impacts: vec![Impact {
					position: GlobalVec3(vec_not_nan!(2., 3., 4.)),
					strength: new_f32!(ImpactStrength(0.5)),
				}],
				..default()
			},
		);

		assert_eq!(
			Some(
				&ShaderBuffer::from(vec![LocalImpact {
					position: Vec3::new(2., 3., 4.),
					strength: 0.5
				}])
				.data
			),
			buffers.get(&material.impacts).map(|b| &b.data)
		);
	}

	#[test]
	fn set_new_impacts_when_assets_data_missing() {
		let mut buffers = Assets::default();
		let mut material = EffectMaterial::default();

		material.update_material(
			&mut buffers,
			&EffectMaterialData {
				impacts: vec![Impact {
					position: GlobalVec3(vec_not_nan!(1., 2., 3.)),
					strength: new_f32!(ImpactStrength(0.5)),
				}],
				..default()
			},
		);
		buffers.remove(&material.impacts);
		material.update_material(
			&mut buffers,
			&EffectMaterialData {
				impacts: vec![Impact {
					position: GlobalVec3(vec_not_nan!(2., 3., 4.)),
					strength: new_f32!(ImpactStrength(0.5)),
				}],
				..default()
			},
		);

		assert_eq!(
			Some(
				&ShaderBuffer::from(vec![LocalImpact {
					position: Vec3::new(2., 3., 4.),
					strength: 0.5
				}])
				.data
			),
			buffers.get(&material.impacts).map(|b| &b.data)
		);
	}
}
