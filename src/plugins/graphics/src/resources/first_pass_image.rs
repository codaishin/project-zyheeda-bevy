use super::window_size::WindowSize;
use crate::traits::resize::Resize;
use bevy::{
	asset::RenderAssetUsages,
	prelude::*,
	render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
};

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct FirstPassImage<TImage = Image>
where
	TImage: Asset + Resize,
{
	pub(crate) handle: Handle<TImage>,
}

impl FirstPassImage {
	pub(crate) fn instantiate(mut images: ResMut<Assets<Image>>, mut commands: Commands) {
		let mut image = Image::new_fill(
			Extent3d::default(),
			TextureDimension::D2,
			&[0, 0, 0, 255],
			TextureFormat::Bgra8UnormSrgb,
			RenderAssetUsages::default(),
		);
		image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
			| TextureUsages::RENDER_ATTACHMENT
			| TextureUsages::COPY_SRC;

		commands.insert_resource(Self {
			handle: images.add(image),
		});
	}
}

impl<TImage> FirstPassImage<TImage>
where
	TImage: Asset + Resize,
{
	pub(crate) fn update_size(
		first_pass_image: Res<Self>,
		window_size: Res<WindowSize>,
		mut images: ResMut<Assets<TImage>>,
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

		let Some(image) = images.get_mut(&first_pass_image.handle) else {
			return;
		};

		image.resize(Extent3d {
			width,
			height,
			depth_or_array_layers,
		});
	}
}

#[cfg(test)]
mod test {
	use std::{
		ops::DerefMut,
		sync::{Arc, Mutex},
	};

	use super::*;
	use crate::resources::window_size::WindowSize;
	use common::{
		is_changed_resource,
		test_tools::utils::SingleThreadedApp,
		traits::nested_mock::NestedMocks,
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Asset, TypePath, NestedMocks)]
	struct _Image {
		mock: Mock_Image,
	}

	#[automock]
	impl Resize for _Image {
		fn resize(&mut self, size: Extent3d) {
			self.mock.resize(size);
		}
	}

	fn setup(image: _Image) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut images = Assets::<_Image>::default();
		let first_pass_image = FirstPassImage {
			handle: images.add(image),
		};
		app.insert_resource(images);
		app.insert_resource(first_pass_image);
		app.add_systems(Update, FirstPassImage::<_Image>::update_size);

		app
	}

	#[test]
	fn update_image_size() {
		let mut app = setup(_Image::new().with_mock(assert));
		app.insert_resource(WindowSize {
			width: 10.1,
			height: 11.9,
		});

		app.update();

		fn assert(image: &mut Mock_Image) {
			image
				.expect_resize()
				.times(1)
				.with(eq(Extent3d {
					width: 10,
					height: 11,
					depth_or_array_layers: 1,
				}))
				.return_const(());
		}
	}

	#[test]
	fn update_image_size_only_once() {
		let mut app = setup(_Image::new().with_mock(assert));
		app.insert_resource(WindowSize {
			width: 10.1,
			height: 11.9,
		});

		app.update();
		app.update();

		fn assert(image: &mut Mock_Image) {
			image.expect_resize().times(1).return_const(());
		}
	}

	#[test]
	fn update_image_size_again_after_window_size_change() {
		let mut app = setup(_Image::new().with_mock(assert));
		app.insert_resource(WindowSize {
			width: 10.1,
			height: 11.9,
		});

		app.update();
		app.world_mut().resource_mut::<WindowSize>().deref_mut();
		app.update();

		fn assert(image: &mut Mock_Image) {
			image.expect_resize().times(2).return_const(());
		}
	}

	#[test]
	fn do_not_update_image_when_width_rounds_to_zero() {
		let mut app = setup(_Image::new().with_mock(assert));
		app.insert_resource(WindowSize {
			width: 0.9,
			height: 11.9,
		});

		app.update();

		fn assert(image: &mut Mock_Image) {
			image.expect_resize().times(0).return_const(());
		}
	}

	#[test]
	fn do_not_change_images_when_width_rounds_to_zero() {
		let changed = Arc::new(Mutex::new(false));
		let mut app = setup(_Image::new().with_mock(|image| {
			image.expect_resize().return_const(());
		}));
		app.add_systems(Last, is_changed_resource!(Assets<_Image>, &changed));
		app.insert_resource(WindowSize {
			width: 1.,
			height: 1.,
		});

		app.update();
		app.insert_resource(WindowSize {
			width: 0.9,
			height: 1.,
		});
		app.update();

		let changed = *changed.lock().unwrap();
		assert!(!changed);
	}

	#[test]
	fn do_not_update_image_when_height_rounds_to_zero() {
		let mut app = setup(_Image::new().with_mock(assert));
		app.insert_resource(WindowSize {
			width: 1.9,
			height: 0.9,
		});

		app.update();

		fn assert(image: &mut Mock_Image) {
			image.expect_resize().times(0).return_const(());
		}
	}

	#[test]
	fn do_not_change_images_when_height_rounds_to_zero() {
		let changed = Arc::new(Mutex::new(false));
		let mut app = setup(_Image::new().with_mock(|image| {
			image.expect_resize().return_const(());
		}));
		app.add_systems(Last, is_changed_resource!(Assets<_Image>, &changed));
		app.insert_resource(WindowSize {
			width: 1.,
			height: 1.,
		});

		app.update();
		app.insert_resource(WindowSize {
			width: 1.,
			height: 0.9,
		});
		app.update();

		let changed = *changed.lock().unwrap();
		assert!(!changed);
	}

	#[test]
	fn do_not_change_images_when_not_changing_window_size() {
		let changed = Arc::new(Mutex::new(false));
		let mut app = setup(_Image::new().with_mock(|image| {
			image.expect_resize().return_const(());
		}));
		app.add_systems(Last, is_changed_resource!(Assets<_Image>, &changed));
		app.insert_resource(WindowSize {
			width: 10.1,
			height: 11.9,
		});

		app.update();
		app.update();

		let changed = *changed.lock().unwrap();
		assert!(!changed);
	}
}
