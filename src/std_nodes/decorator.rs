//! Nodes that have a single child and whose status is some function of the
//! child's status.
use node::Node;
use status::Status;

/// Implements a generic Decorator node
pub struct Decorator<T: Sync>
{
	/// Function that is performed on the child's status
	func: Box<FnMut(&mut T) -> Status>,

	/// Child node
	child: Box<Node<T>>,

	/// Current status of this node
	status: Status, // Is this needed? Why not just use child status?
}
impl<T: Sync> Decorator<T>
{
	/// Creates a new Decorator node with the given child and function
	pub fn new(child: Box<Node<T>>, func: Box<FnMut(&mut T) -> Status>) -> Decorator<T>
	{
		Decorator {
			func: func,
			child: child,
			status: Status::Running,
		}
	}
}
impl<T: Sync> Node<T> for Decorator<T>
{
	fn tick(&mut self, world: &mut T) -> Status
	{
		// First, get the child status
		let child_status = (*self.child).tick(world);

		// Now run it through the function
		let args = (world, );
		self.status = (*self.func).call_mut(args);

		// Return our status
		self.status
	}

	fn reset(&mut self)
	{
		// Reset our own status and then reset our child
		self.status = Status::Running;
		(*self.child).reset();
	}

	fn status(&self) -> Status
	{
		self.status
	}
}
