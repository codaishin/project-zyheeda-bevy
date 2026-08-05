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
#[bind_group_data(LitType)]
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
	pub(crate) lit_type: LitType,
}

impl LitMaterial {
	pub(crate) const SHADER: &str = asset_path!("shaders/lit_shader.wgsl");
	pub(crate) const RANGE: Units = Units::from_u8(15);
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

	pub(crate) fn with_lit_type(mut self, lit_type: LitType) -> Self {
		self.lit_type = lit_type;
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
			lit_type: LitType::Terrain,
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
		let Some(ref mut fragment) = descriptor.fragment else {
			return Ok(());
		};

		let defs: &[&'static str] = match key.bind_group_data {
			LitType::Terrain => &["LIGHT"],
			LitType::DeadSpace => &["NO_LIGHT"],
			LitType::Agent => &["LIGHT", "IGNORE_NDL"],
		};

		fragment
			.shader_defs
			.extend(defs.iter().map(|def| ShaderDefVal::from(*def)));

		Ok(())
	}
}

#[repr(C)]
#[derive(Reflect, Debug, PartialEq, Eq, Hash, Default, Clone, Copy)]
pub(crate) enum LitType {
	#[default]
	Terrain,
	DeadSpace,
	Agent,
}

impl From<&LitMaterial> for LitType {
	fn from(LitMaterial { lit_type, .. }: &LitMaterial) -> Self {
		*lit_type
	}
}
