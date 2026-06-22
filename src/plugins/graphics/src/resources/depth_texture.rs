use crate::resources::window_size::WindowSize;
use bevy::{
	asset::RenderAssetUsages,
	core_pipeline::core_3d::graph::{Core3d, Node3d},
	ecs::query::QueryItem,
	image::{ImageCompareFunction, ImageSampler, ImageSamplerDescriptor},
	prelude::*,
	render::{
		RenderApp,
		extract_component::{ExtractComponent, ExtractComponentPlugin},
		extract_resource::{ExtractResource, ExtractResourcePlugin},
		render_asset::RenderAssets,
		render_graph::{
			NodeRunError,
			RenderGraphContext,
			RenderGraphExt,
			RenderLabel,
			ViewNode,
			ViewNodeRunner,
		},
		render_resource::{
			CommandEncoderDescriptor,
			Extent3d,
			Origin3d,
			TexelCopyTextureInfo,
			TextureAspect,
			TextureDimension,
			TextureFormat,
			TextureUsages,
		},
		renderer::RenderContext,
		texture::GpuImage,
		view::ViewDepthTexture,
	},
};
use common::{
	error_logger::{GlobalErrorLogger, Log},
	errors::{ErrorData, Level},
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

		let Some(image) = images.get_mut(&depth.handle) else {
			return;
		};

		image.resize(Extent3d {
			width,
			height,
			depth_or_array_layers,
		});
	}
}

#[derive(RenderLabel, Debug, PartialEq, Eq, Hash, Default)]
pub(crate) struct DepthTextureLabel<TPass>(PhantomData<TPass>);

impl<TPass> Clone for DepthTextureLabel<TPass> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<TPass> Copy for DepthTextureLabel<TPass> {}

impl<TPass> DepthTextureLabel<TPass> {
	pub(crate) const fn new() -> Self {
		Self(PhantomData)
	}
}

#[derive(Default)]
pub(crate) struct CopyDepthTextureNode<T>(PhantomData<T>);

impl<T> CopyDepthTextureNode<T> {
	fn log(error: CopyDepthTextureError) {
		GlobalErrorLogger::INSTANCE.log(error);
	}

	fn dimensions_ok(
		src: &ViewDepthTexture,
		dst: &GpuImage,
	) -> Result<(), (TextureDimensions, TextureDimensions)> {
		let src_extend = TextureDimensions {
			width: src.texture.width(),
			height: src.texture.height(),
		};

		let dst_extend = TextureDimensions {
			width: dst.size.width,
			height: dst.size.height,
		};

		if src_extend != dst_extend {
			return Err((src_extend, dst_extend));
		}

		Ok(())
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct TextureDimensions {
	width: u32,
	height: u32,
}

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
		let label = DepthTextureLabel::<TCamera>::new();
		let edges = (Node3d::EndPrepasses, label, Node3d::MainOpaquePass);

		self.add_plugins((
			ExtractResourcePlugin::<DepthTexture<TCamera>>::default(),
			ExtractComponentPlugin::<TCamera>::default(),
		))
		.add_systems(Startup, DepthTexture::<TCamera>::instantiate)
		.add_systems(
			First,
			DepthTexture::<TCamera>::update_size.after(WindowSize::update),
		);

		self.sub_app_mut(RenderApp)
			.add_render_graph_node::<ViewNodeRunner<CopyDepthTextureNode<TCamera>>>(Core3d, label)
			.add_render_graph_edges(Core3d, edges);

		self
	}
}

impl<T> ViewNode for CopyDepthTextureNode<T>
where
	T: Component,
{
	type ViewQuery = (&'static ViewDepthTexture, &'static T);

	fn run<'w>(
		&self,
		_: &mut RenderGraphContext,
		render_context: &mut RenderContext<'w>,
		(src, ..): QueryItem<'w, '_, Self::ViewQuery>,
		world: &'w World,
	) -> Result<(), NodeRunError> {
		let Some(depth_image) = world.get_resource::<DepthTexture<T>>() else {
			Self::log(CopyDepthTextureError::NoSourceImage);
			return Ok(());
		};
		let Some(images) = world.get_resource::<RenderAssets<GpuImage>>() else {
			Self::log(CopyDepthTextureError::NoGpuImages);
			return Ok(());
		};
		let Some(dst) = images.get(&depth_image.handle) else {
			Self::log(CopyDepthTextureError::NoDestinationImage);
			return Ok(());
		};

		if let Err((src, dst)) = Self::dimensions_ok(src, dst) {
			Self::log(CopyDepthTextureError::DimensionMismatch { src, dst });
			return Ok(());
		}

		render_context.add_command_buffer_generation_task(move |render_device| {
			let mut command_encoder =
				render_device.create_command_encoder(&CommandEncoderDescriptor {
					label: Some("Copy Depth Texture"),
				});
			command_encoder.copy_texture_to_texture(
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
				dst.size,
			);
			command_encoder.finish()
		});

		Ok(())
	}
}

#[derive(Debug, PartialEq)]
enum CopyDepthTextureError {
	NoSourceImage,
	NoDestinationImage,
	NoGpuImages,
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
			CopyDepthTextureError::NoGpuImages => {
				write!(f, "gpu images missing")
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
