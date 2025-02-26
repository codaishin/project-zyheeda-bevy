use super::ReversAble;

impl<T> ReversAble for Vec<T> {
	fn reverse_collection(&mut self) {
		self.reverse();
	}
}
