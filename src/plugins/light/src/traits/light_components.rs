use bevy::prelude::*;

pub(crate) trait LightComponent: Component {
	fn intensity_mut(&mut self) -> &mut f32;
}

impl LightComponent for PointLight {
	fn intensity_mut(&mut self) -> &mut f32 {
		&mut self.intensity
	}
}

impl LightComponent for SpotLight {
	fn intensity_mut(&mut self) -> &mut f32 {
		&mut self.intensity
	}
}

impl LightComponent for DirectionalLight {
	fn intensity_mut(&mut self) -> &mut f32 {
		&mut self.illuminance
	}
}
