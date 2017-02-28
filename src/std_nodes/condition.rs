//! Nodes whose status is determined by a function
use node::Node;
use status::Status;

/// A node whose success depends on a function that can be run in a single tick
pub struct Condition<T: Sync>
{
	/// Function that is performed to determine the node's status
	///
	/// A return value of `true` means success and a return value of `false` means failure
	func: Box<FnMut(&mut T) -> bool>,

	/// Return status of the last tick
	status: Status,
}
impl<T: Sync> Condition<T>
{
	/// Constructs a new Condition node
	///
	/// If the functio returns `true`, then then node succeeds. Otherwise the node fails.
	pub fn new(func: Box<FnMut(&mut T) -> bool>) -> Condition<T>
	{
		Condition {
			func: func,
			status: Status::Running,
		}
	}
}
impl<T: Sync> Node<T> for Condition<T>
{
	fn tick(&mut self, world: &mut T) -> Status
	{
		self.status = if (*self.func)(world) {
			Status::Succeeded
		} else {
			Status::Failed
		};

		self.status
	}

	fn reset(&mut self)
	{
		// No-op
	}

	fn status(&self) -> Status
	{
		self.status
	}
}
