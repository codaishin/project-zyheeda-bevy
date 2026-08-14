use crate::{
	components::{camera_labels::WorldPass, distance_texture::DistanceTexture},
	materials::lit_material::LitMaterial,
	resources::los_image::{LoSImageCubemap, LoSImageShared},
};
use bevy::{
	asset::RenderAssetUsages,
	camera::{RenderTarget, visibility::RenderLayers},
	ecs::{entity::EntityHashSet, system::StaticSystemParam},
	image::{ImageSampler, TextureFormatPixelInfo},
	prelude::*,
	render::{
		extract_component::ExtractComponent,
		render_resource::{
			Extent3d,
			TextureDescriptor,
			TextureDimension,
			TextureFormat,
			TextureUsages,
			TextureViewDescriptor,
			TextureViewDimension,
		},
	},
};
use common::prelude::*;

#[derive(Component, Debug, PartialEq, Default)]
#[relationship_target(relationship = LoSCameraOf, linked_spawn)]
#[require(Transform)]
pub(crate) struct LoSCameras(EntityHashSet);

#[derive(Component, Debug, PartialEq)]
#[relationship(relationship_target = LoSCameras)]
#[require(Transform)]
pub(crate) struct LoSCameraOf(pub(crate) Entity);

/// Basis for LoS cubemap assuming the following layout
///
/// # As Atlas
/// ```css
///     ┌───┐
///     │ U │
/// ┌───┼───┼───┬───┐
/// │ L │ F │ R │ B │
/// └───┼───┼───┴───┘
///     │ D │
///     └───┘
/// ```
///
/// # Cubemap layers
/// 0. [`LoS::Right`]
/// 1. [`LoS::Left`]
/// 2. [`LoS::Up`]
/// 3. [`LoS::Down`]
/// 4. [`LoS::Forward`]
/// 5. [`LoS::Backward`]
///
#[derive(Component, ExtractComponent, Debug, PartialEq, Clone, Copy)]
#[component(immutable)]
#[require(
	Camera3d,
	Msaa::Off,
	Projection::Perspective(PerspectiveProjection {
		aspect_ratio: 1.,
		fov: 90_f32.to_radians(),
		far: *LitMaterial::RANGE,
		..default()
	})
)]
pub(crate) enum LoS {
	Right,
	Left,
	Up,
	Down,
	Forward,
	Backward,
}

impl LoS {
	pub(crate) const SIDE: u32 = 1024;
	pub(crate) const FMT: TextureFormat = TextureFormat::R32Float;

	fn looking_to(&self) -> LookingTo {
		match self {
			LoS::Right => LookingTo {
				direction: Dir3::X,
				up: Dir3::Y,
			},
			LoS::Left => LookingTo {
				direction: Dir3::NEG_X,
				up: Dir3::Y,
			},
			LoS::Up => LookingTo {
				direction: Dir3::Y,
				up: Dir3::Z,
			},
			LoS::Down => LookingTo {
				direction: Dir3::NEG_Y,
				up: Dir3::NEG_Z,
			},
			LoS::Forward => LookingTo {
				direction: Dir3::NEG_Z,
				up: Dir3::Y,
			},
			LoS::Backward => LookingTo {
				direction: Dir3::Z,
				up: Dir3::Y,
			},
		}
	}

	pub(crate) fn cubemap_depth(&self) -> u32 {
		match self {
			LoS::Right => 0,
			LoS::Left => 1,
			LoS::Up => 2,
			LoS::Down => 3,
			LoS::Forward => 4,
			LoS::Backward => 5,
		}
	}

	fn order(&self) -> isize {
		match self {
			LoS::Right => -6,
			LoS::Left => -5,
			LoS::Up => -4,
			LoS::Down => -3,
			LoS::Forward => -2,
			LoS::Backward => -1,
		}
	}

	pub(crate) fn sub_view_data_size() -> Option<usize> {
		let pixel_size = LoS::FMT.pixel_size().ok()?;

		Some(pixel_size * (LoS::SIDE * LoS::SIDE) as usize)
	}

	pub(crate) fn init_shared_depth(
		mut commands: ZyheedaCommands,
		mut images: ResMut<Assets<Image>>,
	) {
		let mut image = Image::new_target_texture(LoS::SIDE, LoS::SIDE, LoS::FMT, None);

		image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
		image.texture_descriptor.usage &= !TextureUsages::COPY_DST;
		image.sampler = ImageSampler::nearest();

		commands.insert_resource(LoSImageShared {
			handle: images.add(image),
		});
	}

	pub(crate) fn init_cubemap(mut commands: ZyheedaCommands, mut images: ResMut<Assets<Image>>) {
		let usage = TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST;
		let size = Extent3d {
			width: LoS::SIDE,
			height: LoS::SIDE,
			depth_or_array_layers: 6,
		};
		let data = LoS::sub_view_data_size()
			.map(|view_size| vec![0; view_size * size.depth_or_array_layers as usize]);

		commands.insert_resource(LoSImageCubemap {
			handle: images.add(Image {
				data,
				texture_descriptor: TextureDescriptor {
					size,
					format: LoS::FMT,
					dimension: TextureDimension::D2,
					label: None,
					mip_level_count: 1,
					sample_count: 1,
					usage,
					view_formats: &[],
				},
				texture_view_descriptor: Some(TextureViewDescriptor {
					dimension: Some(TextureViewDimension::Cube),
					base_array_layer: 0,
					array_layer_count: Some(6),
					..default()
				}),
				asset_usage: RenderAssetUsages::default(),
				copy_on_resize: true,
				sampler: ImageSampler::nearest(),
				..default()
			}),
		});
	}
}

impl Prefab<()> for LoS {
	type TError = Unreachable;
	type TSystemParam = Res<'static, LoSImageShared>;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		shared: StaticSystemParam<Self::TSystemParam>,
	) -> Result<(), Self::TError> {
		let LookingTo { direction, up } = self.looking_to();

		entity.try_insert((
			RenderTarget::None {
				size: UVec2 {
					x: LoS::SIDE,
					y: LoS::SIDE,
				},
			},
			DistanceTexture(shared.handle.clone()),
			RenderLayers::from(WorldPass),
			Camera {
				order: self.order(),
				..default()
			},
			Transform::default().looking_to(direction, up),
		));

		Ok(())
	}
}

struct LookingTo {
	direction: Dir3,
	up: Dir3,
}
