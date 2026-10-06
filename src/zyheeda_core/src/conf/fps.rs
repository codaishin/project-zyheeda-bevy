use macros::InRange;

#[derive(InRange)]
#[in_range(low = 30, high = 60)]
pub struct DisplayFPS(u8);

#[macro_export]
macro_rules! display_fps {
	($fps:literal) => {{
		use $crate::conf::fps::DisplayFPS;

		const FPS: DisplayFPS = match DisplayFPS::try_new($fps) {
			Ok(fps) => fps,
			Err(_) => panic!("Out of allowed range [30, 60]"),
		};

		FPS
	}};
}

pub use display_fps;

#[derive(InRange)]
#[in_range(low = 30, high = 240)]
pub struct PhysicsFPS(u8);

#[macro_export]
macro_rules! physics_fps {
	($fps:literal) => {{
		use $crate::conf::fps::PhysicsFPS;

		const FPS: PhysicsFPS = match PhysicsFPS::try_new($fps) {
			Ok(fps) => fps,
			Err(_) => panic!("Out of allowed range [30, 240]"),
		};

		FPS
	}};
}

pub use physics_fps;
