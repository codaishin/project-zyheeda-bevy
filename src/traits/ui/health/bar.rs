use crate::{
	components::{Bar, Health},
	traits::ui::UIBarUpdate,
};

impl UIBarUpdate<Health> for Bar<Health> {
	fn update(&mut self, value: &Health) {
		self.current = value.current as f32;
		self.max = value.max as f32;
	}
}
