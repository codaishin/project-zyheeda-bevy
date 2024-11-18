pub enum AnimationPriority {
	High,
	Medium,
	Low,
}

pub trait StartAnimation<TAnimation> {
	fn start_animation<TLayer>(&mut self, layer: TLayer, animation: TAnimation)
	where
		TLayer: Into<AnimationPriority> + 'static;
}

pub trait StopAnimation {
	fn stop_animation<TLayer>(&mut self, layer: TLayer)
	where
		TLayer: Into<AnimationPriority> + 'static;
}
