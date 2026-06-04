use bevy::prelude::*;

pub trait UiCamera {
	type TUiCamera: Component;
}

pub trait FirstPassCamera {
	type TFirstPassCamera: Component;
}

pub trait WorldCameras {
	type TWorldCameras: Component;
}
