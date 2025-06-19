use super::*;
use common::traits::spawn::Spawn;
use std::{
	marker::PhantomData,
	ops::DerefMut,
	sync::{Arc, Mutex},
	time::Instant,
};

pub(super) fn init(app: &mut App) {
	let fps = Fps::<()>::new(u32::MAX);
	let fps_fixed = Fps::<Fixed>::new(u32::MAX);
	let fps_render = Fps::<Render>::new(u32::MAX);

	app.insert_resource(fps)
		.insert_resource(fps_fixed)
		.insert_resource(fps_render.clone())
		.add_systems(Startup, DisplayFps::spawn)
		.add_systems(Last, DisplayFps::update)
		.add_systems(Update, MeasureFps::system::<()>)
		.add_systems(FixedUpdate, MeasureFps::system::<Fixed>);

	let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
		return;
	};

	render_app
		.insert_resource(fps_render)
		.add_systems(Render, MeasureFps::system::<Render>);
}

#[derive(Resource, Debug, Clone)]
struct Fps<T = ()> {
	fps: Arc<Mutex<u32>>,
	phantom_data: PhantomData<T>,
}

impl<T> Fps<T> {
	fn new(fps: u32) -> Self {
		Self {
			fps: Arc::new(Mutex::new(fps)),
			phantom_data: PhantomData,
		}
	}
}

#[derive(Debug, PartialEq)]
struct MeasureFps {
	timer: Instant,
	counter: u32,
}

impl Default for MeasureFps {
	fn default() -> Self {
		Self {
			timer: Instant::now(),
			counter: 0,
		}
	}
}

impl MeasureFps {
	const FRAME_DURATION: Duration = Duration::from_secs(1);

	fn system<T>(mut measure: Local<MeasureFps>, fps: ResMut<Fps<T>>)
	where
		T: Sync + Send + 'static,
	{
		let measure = measure.deref_mut();
		measure.counter += 1;

		if measure.timer.elapsed() < Self::FRAME_DURATION {
			return;
		}

		let Ok(mut fps) = fps.fps.try_lock() else {
			return;
		};

		*fps = measure.counter;
		*measure = MeasureFps::default();
	}
}

#[derive(Component, Debug, PartialEq, Default)]
#[require(Node = DisplayFps::top_left(), Text)]
struct DisplayFps;

impl DisplayFps {
	fn top_left() -> Node {
		Node {
			position_type: PositionType::Absolute,
			left: Val::Px(10.),
			top: Val::Px(10.),
			width: Val::Px(500.),
			height: Val::Px(20.),
			..default()
		}
	}

	fn update(
		mut displays: Query<&mut Text, With<DisplayFps>>,
		fps: Res<Fps>,
		fps_fixed: Res<Fps<Fixed>>,
		fps_render: Res<Fps<Render>>,
	) {
		let Ok(mut text) = displays.single_mut() else {
			return;
		};

		let (Ok(fps), Ok(fps_fixed), Ok(fps_render)) = (
			fps.fps.try_lock(),
			fps_fixed.fps.try_lock(),
			fps_render.fps.try_lock(),
		) else {
			return;
		};

		*text = Text(format!(
			"FPS: {fps} (fixed: {fps_fixed}, render: {fps_render})"
		));
	}
}
