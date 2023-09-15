mod behavior_scheduler;

pub trait Add<T> {
	fn add(&mut self, value: T);
}
