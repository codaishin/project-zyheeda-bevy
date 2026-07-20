use crate::resources::window_size::WindowSize;
use bevy::{
	asset::RenderAssetUsages,
	core_pipeline::{Core3d, Core3dSystems},
	image::{ImageCompareFunction, ImageSampler, ImageSamplerDescriptor},
	prelude::*,
	render::{
		RenderApp,
		extract_component::{ExtractComponent, ExtractComponentPlugin},
		extract_resource::{ExtractResource, ExtractResourcePlugin},
		render_asset::RenderAssets,
		render_resource::{
			Extent3d,
			Origin3d,
			TexelCopyTextureInfo,
			TextureAspect,
			TextureDimension,
			TextureFormat,
			TextureUsages,
		},
		renderer::{RenderContext, ViewQuery},
		texture::GpuImage,
		view::ViewDepthTexture,
	},
};
use common::{
	errors::{ErrorData, Level},
	systems::log::OnError,
	traits::thread_safe::ThreadSafe,
};
use std::{
	fmt::{Debug, Display},
	hash::Hash,
	marker::PhantomData,
	time::Duration,
};

#[derive(Resource, ExtractResource, Debug, PartialEq, Clone)]
pub(crate) struct DepthTexture<TPass>
where
	TPass: ThreadSafe,
{
	pub(crate) handle: Handle<Image>,
	_p: PhantomData<TPass>,
}

impl<TPass> DepthTexture<TPass>
where
	TPass: ThreadSafe,
{
	pub(crate) fn instantiate(mut c: Commands, mut images: ResMut<Assets<Image>>) {
		let mut image = Image::new_uninit(
			Extent3d::default(),
			TextureDimension::D2,
			TextureFormat::Depth32Float,
			RenderAssetUsages::default(),
		);
		image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST;
		image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
			label: Some("comparison sampler".to_owned()),
			compare: Some(ImageCompareFunction::Always),
			..default()
		});

		c.insert_resource(Self {
			handle: images.add(image),
			_p: PhantomData,
		});
	}

	pub(crate) fn update_size(
		depth: Res<Self>,
		window_size: Res<WindowSize>,
		mut images: ResMut<Assets<Image>>,
	) {
		if !window_size.is_changed() {
			return;
		}

		let width = window_size.width as u32;
		let height = window_size.height as u32;
		let depth_or_array_layers = 1;

		if width == 0 || height == 0 {
			return;
		}

		let Some(mut image) = images.get_mut(&depth.handle) else {
			return;
		};

		image.resize(Extent3d {
			width,
			height,
			depth_or_array_layers,
		});
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct TextureDimensions(UVec2);

pub(crate) trait CopyDepthTexture {
	fn copy_depth_texture<TCamera>(&mut self) -> &mut Self
	where
		TCamera: Component + ExtractComponent + Debug + Default + Clone + Eq + Hash;
}

impl CopyDepthTexture for App {
	fn copy_depth_texture<TCamera>(&mut self) -> &mut Self
	where
		TCamera: Component + ExtractComponent + Debug + Default + Clone + Eq + Hash,
	{
		self.add_plugins((
			ExtractResourcePlugin::<DepthTexture<TCamera>>::default(),
			ExtractComponentPlugin::<TCamera>::default(),
		))
		.add_systems(Startup, DepthTexture::<TCamera>::instantiate)
		.add_systems(
			First,
			DepthTexture::<TCamera>::update_size.after(WindowSize::update),
		);

		self.sub_app_mut(RenderApp).add_systems(
			Core3d,
			copy_texture_system::<TCamera>
				.pipe(OnError::log)
				.after(Core3dSystems::Prepass)
				.before(Core3dSystems::MainPass),
		);

		self
	}
}

fn dimensions_ok(
	src: &ViewDepthTexture,
	dst: &GpuImage,
) -> Result<(), (TextureDimensions, TextureDimensions)> {
	let src_extend = TextureDimensions(UVec2 {
		x: src.texture.width(),
		y: src.texture.height(),
	});
	let dst_extend = TextureDimensions(dst.size_2d());

	if src_extend != dst_extend {
		return Err((src_extend, dst_extend));
	}

	Ok(())
}

fn copy_texture_system<T>(
	view: ViewQuery<&ViewDepthTexture, With<T>>,
	depth_image: Option<Res<DepthTexture<T>>>,
	images: Res<RenderAssets<GpuImage>>,
	mut render_context: RenderContext,
) -> Result<(), CopyDepthTextureError>
where
	T: Component,
{
	let src = view.into_inner();

	let Some(depth_image) = depth_image else {
		return Err(CopyDepthTextureError::NoSourceImage);
	};
	let Some(dst) = images.get(&depth_image.handle) else {
		return Err(CopyDepthTextureError::NoDestinationImage);
	};

	if let Err((src, dst)) = dimensions_ok(src, dst) {
		return Err(CopyDepthTextureError::DimensionMismatch { src, dst });
	};

	render_context.command_encoder().copy_texture_to_texture(
		TexelCopyTextureInfo {
			texture: &src.texture,
			mip_level: 0,
			origin: Origin3d::default(),
			aspect: TextureAspect::DepthOnly,
		},
		TexelCopyTextureInfo {
			texture: &dst.texture,
			mip_level: 0,
			origin: Origin3d::default(),
			aspect: TextureAspect::DepthOnly,
		},
		src.texture.size(),
	);

	Ok(())
}

#[derive(Debug, PartialEq)]
enum CopyDepthTextureError {
	NoSourceImage,
	NoDestinationImage,
	DimensionMismatch {
		src: TextureDimensions,
		dst: TextureDimensions,
	},
}

impl Display for CopyDepthTextureError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			CopyDepthTextureError::NoSourceImage => {
				write!(f, "no source image")
			}
			CopyDepthTextureError::NoDestinationImage => {
				write!(f, "no destination image")
			}
			CopyDepthTextureError::DimensionMismatch { src, dst } => {
				write!(f, "dimensions differ: src: {src:?}, dst: {dst:?}")
			}
		}
	}
}

impl ErrorData for CopyDepthTextureError {
	fn rate_limit() -> Option<std::time::Duration> {
		Some(Duration::from_secs(2))
	}

	fn level(&self) -> Level {
		Level::Warning
	}

	fn label() -> impl Display {
		"Copy Depth Texture Error"
	}

	fn into_details(self) -> impl Display {
		self
	}
}
