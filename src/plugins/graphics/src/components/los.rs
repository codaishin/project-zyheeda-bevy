use crate::{
	components::camera_labels::WorldPass,
	materials::lit_material::LitMaterial,
	resources::los_image::{LoSImageAtlas, LoSImageCubemap},
};
use bevy::{
	asset::RenderAssetUsages,
	camera::{ImageRenderTarget, RenderTarget, Viewport, visibility::RenderLayers},
	ecs::{entity::EntityHashSet, system::StaticSystemParam},
	image::TextureFormatPixelInfo,
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
use common::{
	errors::Unreachable,
	traits::prefab::{Prefab, PrefabEntityCommands},
	zyheeda_commands::ZyheedaCommands,
};

#[derive(Component, Debug, PartialEq, Default)]
#[relationship_target(relationship = LoSCameraOf, linked_spawn)]
#[require(Transform)]
pub(crate) struct LoSCameras(EntityHashSet);

#[derive(Component, Debug, PartialEq)]
#[relationship(relationship_target = LoSCameras)]
#[require(Transform)]
pub(crate) struct LoSCameraOf(pub(crate) Entity);

/// Basis for LoS cubemap using the following atlas layout
/// ```css
///     ┌───┐
///     │ U │
/// ┌───┼───┼───┬───┐
/// │ L │ F │ R │ B │
/// └───┼───┼───┴───┘
///     │ D │
///     └───┘
/// ```
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
	pub(crate) const SUB_VIEW: u32 = 1024;
	pub(crate) const FULL_WIDTH: u32 = LoS::SUB_VIEW * 4;
	pub(crate) const FULL_HEIGHT: u32 = LoS::SUB_VIEW * 3;

	const FMT: TextureFormat = TextureFormat::Rgba32Float;

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

	pub(crate) fn atlas_offset(&self) -> UVec2 {
		match self {
			LoS::Right => UVec2 {
				x: LoS::SUB_VIEW * 2,
				y: LoS::SUB_VIEW,
			},
			LoS::Left => UVec2 {
				x: 0,
				y: LoS::SUB_VIEW,
			},
			LoS::Up => UVec2 {
				x: LoS::SUB_VIEW,
				y: 0,
			},
			LoS::Down => UVec2 {
				x: LoS::SUB_VIEW,
				y: LoS::SUB_VIEW * 2,
			},
			LoS::Forward => UVec2 {
				x: LoS::SUB_VIEW,
				y: LoS::SUB_VIEW,
			},
			LoS::Backward => UVec2 {
				x: LoS::SUB_VIEW * 3,
				y: LoS::SUB_VIEW,
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

		Some(pixel_size * (LoS::SUB_VIEW * LoS::SUB_VIEW) as usize)
	}

	pub(crate) fn init_atlas(mut commands: ZyheedaCommands, mut images: ResMut<Assets<Image>>) {
		let mut image =
			Image::new_target_texture(LoS::FULL_WIDTH, LoS::FULL_HEIGHT, LoS::FMT, None);

		image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
		image.texture_descriptor.usage &= !TextureUsages::COPY_DST;

		commands.insert_resource(LoSImageAtlas {
			handle: images.add(image),
		});
	}

	pub(crate) fn init_cubemap(mut commands: ZyheedaCommands, mut images: ResMut<Assets<Image>>) {
		let usage = TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST;
		let size = Extent3d {
			width: LoS::SUB_VIEW,
			height: LoS::SUB_VIEW,
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
				..default()
			}),
		});
	}
}

impl Prefab<()> for LoS {
	type TError = Unreachable;
	type TSystemParam = Res<'static, LoSImageAtlas>;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		los_image: StaticSystemParam<Self::TSystemParam>,
	) -> Result<(), Self::TError> {
		let LookingTo { direction, up } = self.looking_to();

		entity.try_insert((
			RenderTarget::Image(ImageRenderTarget {
				handle: los_image.handle.clone(),
				scale_factor: 1.,
			}),
			RenderLayers::from(WorldPass),
			Camera {
				order: self.order(),
				viewport: Some(Viewport {
					physical_position: self.atlas_offset(),
					physical_size: UVec2 {
						x: LoS::SUB_VIEW,
						y: LoS::SUB_VIEW,
					},
					..default()
				}),
				clear_color: ClearColorConfig::Custom(Color::WHITE),
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
