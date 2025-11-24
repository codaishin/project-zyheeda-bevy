use std::slice::Iter;

pub(crate) fn leak_iterator<T>(animations: Vec<T>) -> Iter<'static, T> {
	Box::new(animations).leak().iter()
}
