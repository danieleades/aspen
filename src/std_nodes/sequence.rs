//! Nodes that have children and tick them in a sequential order
//!
//! NOTE: There is no Sequence* node, since the choice of not having the nodes
//! automatically reset causes a normal Sequence node to have the same behavior
//! as a Sequence*.

use node::Node;
use status::Status;

/// Implements a Sequence node
///
/// This node will tick all of its children in order until one of them returns
/// either `Status::Running` or `Status::Failed`. If none do, this node succeeds.
pub struct Sequence<T: Sync>
{
	/// Vector containing the children of this node
	children: Vec<Box<Node<T>>>,

	/// Current status of the node
	status: Status, // Do we really need this? Why not tick current child?
}
impl<T: Sync> Sequence<T>
{
	/// Creates a new Sequence* node from a vector of Nodes
	pub fn new(children: Vec<Box<Node<T>>>) -> Sequence<T>
	{
		Sequence {
			children: children,
			status: Status::Running,
		}
	}
}
impl<T: Sync> Node<T> for Sequence<T>
{
	fn tick(&mut self, world: &mut T) -> Status
	{
		// Tick the children in order
		for ptr in self.children.iter_mut() {
			// First, tick the current child to get its status
			let child_status = (*ptr).tick(world);

			// Now, set our status to that of the current child
			self.status = child_status;

			// Then decide if we're done ticking based on our children
			if child_status != Status::Succeeded {
				return child_status;
			}
		}

		// Do a sanity check
		assert_eq!(self.status, Status::Succeeded);

		// Return that we succeeded
		Status::Succeeded
	}

	fn reset(&mut self)
	{
		// Reset our own status
		self.status = Status::Running;

		// Then we reset all of our children
		for ptr in self.children.iter_mut() {
			(*ptr).reset();
		}
	}

	fn status(&self) -> Status
	{
		self.status
	}
}
