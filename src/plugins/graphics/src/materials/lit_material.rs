use bevy::{
	material::{descriptor::RenderPipelineDescriptor, specialize::SpecializedMeshPipelineError},
	mesh::MeshVertexBufferLayoutRef,
	pbr::{ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline},
	prelude::*,
	render::render_resource::AsBindGroup,
	shader::{ShaderDefVal, ShaderRef},
};
use common::tools::Units;
use macros::asset_path;

pub(crate) type StandardLitMaterial = ExtendedMaterial<StandardMaterial, LitMaterial>;

#[derive(Asset, AsBindGroup, Reflect, Debug, PartialEq, Clone)]
#[bind_group_data(DeadSpace)]
pub(crate) struct LitMaterial {
	#[uniform(100)]
	pub(crate) player_position: Vec3,
	#[uniform(101)]
	range: f32,
	#[uniform(102)]
	min_light: f32,
	#[texture(103, dimension = "cube")]
	#[sampler(104)]
	pub(crate) los: Handle<Image>,
	pub(crate) dead_space: bool,
}

impl LitMaterial {
	pub(crate) const SHADER: &str = asset_path!("shaders/lit_shader.wgsl");
	pub(crate) const RANGE: Units = Units::from_u8(10);
	pub(crate) const MIN_LIGHT: f32 = 0.01;

	pub(crate) fn from_player_position(player_position: Vec3) -> Self {
		Self {
			player_position,
			..default()
		}
	}

	pub(crate) fn with_los_cubemap(mut self, los: Handle<Image>) -> Self {
		self.los = los;
		self
	}

	pub(crate) fn with_dead_space(mut self, DeadSpace(dead_space): DeadSpace) -> Self {
		self.dead_space = dead_space;
		self
	}
}

impl Default for LitMaterial {
	fn default() -> Self {
		Self {
			player_position: Vec3::ZERO,
			range: *Self::RANGE,
			min_light: Self::MIN_LIGHT,
			los: Handle::default(),
			dead_space: false,
		}
	}
}

impl MaterialExtension for LitMaterial {
	fn fragment_shader() -> ShaderRef {
		Self::SHADER.into()
	}

	fn deferred_fragment_shader() -> ShaderRef {
		Self::SHADER.into()
	}

	fn specialize(
		_: &MaterialExtensionPipeline,
		descriptor: &mut RenderPipelineDescriptor,
		_: &MeshVertexBufferLayoutRef,
		key: MaterialExtensionKey<Self>,
	) -> Result<(), SpecializedMeshPipelineError> {
		if let (DeadSpace(true), Some(fragment)) = (key.bind_group_data, &mut descriptor.fragment) {
			fragment.shader_defs.push(ShaderDefVal::from("DEAD_SPACE"));
		}

		Ok(())
	}
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct DeadSpace(pub(crate) bool);

impl From<&LitMaterial> for DeadSpace {
	fn from(LitMaterial { dead_space, .. }: &LitMaterial) -> Self {
		Self(*dead_space)
	}
}
