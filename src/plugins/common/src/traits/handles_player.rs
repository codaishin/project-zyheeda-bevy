use super::{
	accessors::get::{Getter, GetterRefOptional},
	intersect_at::IntersectAt,
};
use crate::tools::{
	collider_info::ColliderInfo,
	movement_animation::MovementAnimation,
	slot_key::SlotKey,
	speed::Speed,
};
use bevy::{math::Ray3d, prelude::*};

pub trait HandlesPlayer {
	type TPlayer: Component;
}

pub trait WithMainCamera {
	type TWithMainCam<TMainCamera>: WithCamera
	where
		TMainCamera: Component;

	fn with_main_camera<TMainCamera>(self) -> Self::TWithMainCam<TMainCamera>
	where
		TMainCamera: Component;
}

pub trait WithCamera {
	type TWithCam<TCamera>: WithCamera
	where
		TCamera: Component;

	fn with_camera<TCamera>(self) -> Self::TWithCam<TCamera>
	where
		TCamera: Component;
}

pub trait PlayerMainCamera {
	type TPlayerMainCamera: Component;
}

pub trait HandlesPlayerCameras {
	type TCamRay: Resource + GetterRefOptional<Ray3d> + IntersectAt;
}

pub trait HandlesPlayerMouse {
	type TMouseHover: Resource + GetterRefOptional<ColliderInfo<Entity>>;
}

pub trait ConfiguresPlayerMovement {
	type TPlayerMovement: Component + Getter<Speed> + GetterRefOptional<MovementAnimation>;
}

pub trait ConfiguresPlayerSkillAnimations {
	type TAnimationMarker: Component;

	fn start_skill_animation(slot_key: SlotKey) -> Self::TAnimationMarker;
	fn stop_skill_animation() -> Self::TAnimationMarker;
}

#[macro_export]
/// Creates a type matching chained calls to:
/// - [`WithMainCamera::with_main_camera`]
/// - [`WithCamera::with_camera`]
///
/// Unfortunately due to the recursive nature of the macro, the types of
/// `with_camera` calls are reversed.
///
/// # Example
///
/// ```
/// use std::marker::PhantomData;
/// use bevy::prelude::Component;
/// use common::{
///   WithCameras,
///   traits::handles_player::{WithCamera, WithMainCamera}
/// };
///
/// type T = WithCameras!(Container, Cam<Main>, Cam<Other2>, Cam<Other1>);
/// let container: T = Container::new()
///   .with_main_camera::<Cam<Main>>()
///   .with_camera::<Cam<Other1>>()
///   .with_camera::<Cam<Other2>>();
///
/// /* --------------------- */
/// /* Dummy implementations */
/// /* --------------------- */
///
/// struct Main;
/// struct Other1;
/// struct Other2;
///
/// #[derive(Component)]
/// struct Cam<T>(PhantomData<T>);
///
/// struct Container<T = ()>(PhantomData<T>);
///
/// impl Container {
///   fn new() -> Self {
///     Container(PhantomData)
///   }
/// }
///
/// impl WithMainCamera for Container {
///   type TWithMainCam<TMainCamera: Component> = Container<(TMainCamera,)>;
///
///   fn with_main_camera<TMainCamera: Component>(self) -> Container<(TMainCamera,)> {
///     Container(PhantomData)
///   }
/// }
///
/// impl<TCameras> WithCamera for Container<TCameras> {
///   type TWithCam<TNewCamera: Component> = Container<(TCameras, TNewCamera)>;
///
///   fn with_camera<TNewCamera: Component>(self) -> Container<(TCameras, TNewCamera)> {
///     Container(PhantomData)
///   }
/// }
/// ```
macro_rules! WithCameras {
	($t_container:ty, $t_main_cam:ty) => {
		<$t_container as WithMainCamera>::TWithMainCam<$t_main_cam>
	};
	($t_container:ty, $t_main_cam:ty, $t_other_cam:ty) => {
		<WithCameras!($t_container, $t_main_cam) as WithCamera>::TWithCam<$t_other_cam>
	};
	($t_container:ty, $t_main_cam:ty, $t_other_cam:ty, $($t_other_cams:ty),+) => {
		<WithCameras!($t_container, $t_main_cam, $($t_other_cams),*) as WithCamera>::TWithCam<$t_other_cam>
	};
}

pub use WithCameras;
