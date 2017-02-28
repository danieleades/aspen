//! Nodes that have children and tick them in a sequential order as long as they
//! fail.
//!
//! NOTE: There is no Selector* node, since the choice of not having the nodes
//! automatically reset causes a normal Selector node to have the same behavior
//! as a Selector*.

use node::Node;
use status::Status;

/// Implements a Selector node
///
/// This node will tick all of its children in order until one of them returns
/// either `Status::Running` or `Status::Success`. If none do, this node fails.
pub struct Selector<T: Sync>
{
	/// Vector containing the children of this node
	children: Vec<Box<Node<T>>>,

	/// Current status of the node
	status: Status, // Do we really need this? Why not tick current child?
}
impl<T: Sync> Selector<T>
{
	/// Creates a new Selector node from a vector of Nodes
	pub fn new(children: Vec<Box<Node<T>>>) -> Selector<T>
	{
		Selector {
			children: children,
			status: Status::Running,
		}
	}
}
impl<T: Sync> Node<T> for Selector<T>
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
			if child_status != Status::Failed {
				return child_status;
			}
		}

		// Do a sanity check
		assert_eq!(self.status, Status::Failed);

		// Return that we failed
		Status::Failed
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
