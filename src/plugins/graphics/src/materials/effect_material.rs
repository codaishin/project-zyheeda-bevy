use crate::{
	components::effect_material_data::{EffectMaterialData, Impact},
	systems::update_material_buffer::UpdateMaterial,
};
use bevy::{
	prelude::*,
	render::{
		render_resource::{AsBindGroup, ShaderType},
		storage::ShaderBuffer,
	},
	shader::ShaderRef,
};
use std::sync::LazyLock;

pub static NO_IMPACTS: LazyLock<Handle<ShaderBuffer>> = LazyLock::new(Handle::default);

#[derive(Asset, TypePath, AsBindGroup, Debug, PartialEq, Clone)]
pub(crate) struct EffectMaterial {
	#[texture(0)]
	#[sampler(1)]
	pub(crate) first_pass: Handle<Image>,
	#[uniform(2)]
	pub(crate) base_color: LinearRgba,
	#[uniform(3)]
	pub(crate) fresnel_color: LinearRgba,
	#[uniform(4)]
	pub(crate) flags: u32,
	#[storage(5, read_only)]
	pub(crate) impacts: Handle<ShaderBuffer>,
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

	pub(crate) fn from_first_pass(first_pass: Handle<Image>) -> Self {
		Self {
			first_pass,
			..default()
		}
	}

	fn compute_impacts(impacts: &[Impact], transform: &GlobalTransform) -> ShaderBuffer {
		let to_global = |Impact { local, strength }: &Impact| {
			let position = transform.transform_point(*local);
			let strength = **strength;

			ImpactEffect { position, strength }
		};

		ShaderBuffer::from(impacts.iter().map(to_global).collect::<Vec<_>>())
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

	fn empty_impacts(&mut self) {
		self.impacts = NO_IMPACTS.clone();
	}

	fn new_impacts(
		&mut self,
		buffers: &mut Assets<ShaderBuffer>,
		impacts: &[Impact],
		transform: &GlobalTransform,
	) {
		self.impacts = buffers.add(Self::compute_impacts(impacts, transform));
	}

	fn updated_impacts(
		&mut self,
		buffers: &mut Assets<ShaderBuffer>,
		impacts: &[Impact],
		transform: &GlobalTransform,
	) {
		if let Some(mut buffer) = buffers.get_mut(&self.impacts) {
			*buffer = Self::compute_impacts(impacts, transform);
		} else {
			self.impacts = buffers.add(Self::compute_impacts(impacts, transform));
		}
	}
}

impl Default for EffectMaterial {
	fn default() -> Self {
		Self {
			first_pass: default(),
			base_color: LinearRgba::from(Self::DEFAULT_COLOR),
			fresnel_color: LinearRgba::from(Self::DEFAULT_FRESNEL),
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

	fn specialize(
		_: &bevy::pbr::MaterialPipeline,
		descriptor: &mut bevy::material::descriptor::RenderPipelineDescriptor,
		_: &bevy::mesh::MeshVertexBufferLayoutRef,
		_: bevy::pbr::MaterialPipelineKey<Self>,
	) -> Result<(), bevy::material::specialize::SpecializedMeshPipelineError> {
		descriptor.primitive.cull_mode = None;

		Ok(())
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

#[derive(Debug, PartialEq, ShaderType)]
pub(crate) struct ImpactEffect {
	position: Vec3,
	strength: f32,
}

impl UpdateMaterial for EffectMaterial {
	type TData = EffectMaterialData;
	type TBuffer = Assets<ShaderBuffer>;

	fn update_material(
		&mut self,
		buffers: &mut Assets<ShaderBuffer>,
		data: &EffectMaterialData,
		transform: &GlobalTransform,
	) {
		let EffectMaterialData { impacts, .. } = data;

		let impacts = impacts.as_slice();

		match impacts {
			[] => self.empty_impacts(),
			_ if self.impacts == *NO_IMPACTS => self.new_impacts(buffers, impacts, transform),
			_ => self.updated_impacts(buffers, impacts, transform),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::effect_material_data::ImpactStrength;
	use zyheeda_core::prelude::*;

	#[test]
	fn set_impacts() {
		let mut buffers = Assets::default();
		let mut material = EffectMaterial::default();

		material.update_material(
			&mut buffers,
			&EffectMaterialData {
				impacts: vec![Impact {
					local: Vec3::new(1., 2., 3.),
					strength: new_f32!(ImpactStrength(0.5)),
				}],
				..default()
			},
			&GlobalTransform::default(),
		);

		assert_eq!(
			Some(
				&ShaderBuffer::from(vec![ImpactEffect {
					position: Vec3::new(1., 2., 3.),
					strength: 0.5
				}])
				.data
			),
			buffers.get(&material.impacts).map(|b| &b.data)
		);
	}

	#[test]
	fn set_impacts_with_regenerated_global_position() {
		let mut buffers = Assets::default();
		let mut material = EffectMaterial::default();

		material.update_material(
			&mut buffers,
			&EffectMaterialData {
				impacts: vec![Impact {
					local: Vec3::new(1., 2., 3.),
					strength: new_f32!(ImpactStrength(0.5)),
				}],
				..default()
			},
			&GlobalTransform::from(Transform::from_xyz(2., 3., 0.).looking_to(Dir3::Z, Dir3::Y)),
		);

		assert_eq!(
			Some(
				&ShaderBuffer::from(vec![ImpactEffect {
					position: Vec3::new(1., 5., -3.),
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
					local: Vec3::new(1., 2., 3.),
					strength: new_f32!(ImpactStrength(0.5)),
				}],
				..default()
			},
			&GlobalTransform::default(),
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
			&GlobalTransform::default(),
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
					local: Vec3::ZERO,
					strength: new_f32!(ImpactStrength(1.0)),
				}],
				..default()
			},
			&GlobalTransform::default(),
		);
		material.update_material(
			&mut buffers,
			&EffectMaterialData {
				impacts: vec![],
				..default()
			},
			&GlobalTransform::default(),
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
					local: Vec3::new(1., 2., 3.),
					strength: new_f32!(ImpactStrength(0.5)),
				}],
				..default()
			},
			&GlobalTransform::default(),
		);

		let first_handle = material.impacts.clone();
		material.update_material(
			&mut buffers,
			&EffectMaterialData {
				impacts: vec![Impact {
					local: Vec3::new(1., 2., 3.),
					strength: new_f32!(ImpactStrength(0.4)),
				}],
				..default()
			},
			&GlobalTransform::default(),
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
					local: Vec3::new(1., 2., 3.),
					strength: new_f32!(ImpactStrength(0.5)),
				}],
				..default()
			},
			&GlobalTransform::default(),
		);

		material.update_material(
			&mut buffers,
			&EffectMaterialData {
				impacts: vec![Impact {
					local: Vec3::new(2., 3., 4.),
					strength: new_f32!(ImpactStrength(0.5)),
				}],
				..default()
			},
			&GlobalTransform::default(),
		);

		assert_eq!(
			Some(
				&ShaderBuffer::from(vec![ImpactEffect {
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
					local: Vec3::new(1., 2., 3.),
					strength: new_f32!(ImpactStrength(0.5)),
				}],
				..default()
			},
			&GlobalTransform::default(),
		);
		buffers.remove(&material.impacts);
		material.update_material(
			&mut buffers,
			&EffectMaterialData {
				impacts: vec![Impact {
					local: Vec3::new(2., 3., 4.),
					strength: new_f32!(ImpactStrength(0.5)),
				}],
				..default()
			},
			&GlobalTransform::default(),
		);

		assert_eq!(
			Some(
				&ShaderBuffer::from(vec![ImpactEffect {
					position: Vec3::new(2., 3., 4.),
					strength: 0.5
				}])
				.data
			),
			buffers.get(&material.impacts).map(|b| &b.data)
		);
	}

	mod flags {
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
}
