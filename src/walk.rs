
/// Helper for tracking parents when recursively walking datasets.
#[derive(Debug)]
pub struct WalkRef<'a, T> {
	pub parent: Option<&'a WalkRef<'a, T>>,
	pub instance: &'a T,
}
