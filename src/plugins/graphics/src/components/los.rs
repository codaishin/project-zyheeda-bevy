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
		far: LitMaterial::MAX_LIGHT_DISTANCE,
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

	fn offset(&self) -> UVec2 {
		match self {
			LoS::Right => UVec2 {
				x: SUB_VIEW * 2,
				y: SUB_VIEW,
			},
			LoS::Left => UVec2 { x: 0, y: SUB_VIEW },
			LoS::Up => UVec2 { x: SUB_VIEW, y: 0 },
			LoS::Down => UVec2 {
				x: SUB_VIEW,
				y: SUB_VIEW * 2,
			},
			LoS::Forward => UVec2 {
				x: SUB_VIEW,
				y: SUB_VIEW,
			},
			LoS::Backward => UVec2 {
				x: SUB_VIEW * 3,
				y: SUB_VIEW,
			},
		}
	}

	fn order(&self) -> isize {
		match self {
			LoS::Right => 101,
			LoS::Left => 102,
			LoS::Up => 103,
			LoS::Down => 104,
			LoS::Forward => 105,
			LoS::Backward => 106,
		}
	}

	pub(crate) fn init_atlas(mut commands: ZyheedaCommands, mut images: ResMut<Assets<Image>>) {
		let image = Image::new_target_texture(
			FULL_WIDTH,
			FULL_HEIGHT,
			TextureFormat::Rgba8Unorm,
			Some(TextureFormat::Rgba8UnormSrgb),
		);

		commands.insert_resource(LoSImageAtlas {
			handle: images.add(image),
		});
	}

	pub(crate) fn init_cubemap(mut commands: ZyheedaCommands, mut images: ResMut<Assets<Image>>) {
		let format = TextureFormat::Rgba8Unorm;
		let usage = TextureUsages::TEXTURE_BINDING
			| TextureUsages::COPY_DST
			| TextureUsages::RENDER_ATTACHMENT;
		let size = Extent3d {
			width: SUB_VIEW,
			height: SUB_VIEW,
			depth_or_array_layers: 6,
		};
		let data = match format.pixel_size() {
			Ok(pixel_size) => Some(vec![
				0;
				pixel_size
					* (size.width * size.height * size.depth_or_array_layers)
						as usize
			]),
			Err(_) => None,
		};

		commands.insert_resource(LoSImageCubemap {
			handle: images.add(Image {
				data,
				texture_descriptor: TextureDescriptor {
					size,
					format,
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

const SUB_VIEW: u32 = 64;
const FULL_WIDTH: u32 = SUB_VIEW * 4;
const FULL_HEIGHT: u32 = SUB_VIEW * 3;

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
					physical_position: self.offset(),
					physical_size: UVec2 {
						x: SUB_VIEW,
						y: SUB_VIEW,
					},
					..default()
				}),
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
