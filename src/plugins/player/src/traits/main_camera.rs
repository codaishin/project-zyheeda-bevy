use bevy::prelude::*;

pub(crate) trait MainCamera {
	type TMainCamera: Component;
}

impl<T> MainCamera for (T,)
where
	T: Component,
{
	type TMainCamera = T;
}

impl<T1, T2> MainCamera for (T1, T2)
where
	T1: MainCamera,
{
	type TMainCamera = T1::TMainCamera;
}
