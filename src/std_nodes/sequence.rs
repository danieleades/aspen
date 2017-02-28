//! Nodes that have children and tick them in a sequential order

use node::Node;
use status::Status;

/// Implements a Sequence* node
pub struct SequencePersistent<T: Sync>
{
	/// Vector containing the children of this node
	children: Vec<Box<Node<T>>>,

	/// Next child to be ticked
	next_tick_index: usize,

	/// Current status of the node
	status: Status,
}
impl<T: Sync> SequencePersistent<T>
{
	/// Creates a new Sequence* node from a vector of Nodes
	pub fn new(children: Vec<Box<Node<T>>>) -> SequencePersistent<T>
	{
		SequencePersistent {
			children: children,
			next_tick_index: 0,
			status: Status::Running,
		}
	}

	/// Returns an immutable reference to the children vector
	pub fn children(&self) -> &Vec<Box<Node<T>>>
	{
		&self.children
	}
}
impl<T: Sync> Node<T> for SequencePersistent<T>
{
	fn tick(&mut self, world: &mut T) -> Status
	{
		for ptr in self.children.iter_mut().skip(self.next_tick_index) {
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

		// If none of them were running or failed, we succeeded. This means
		// that we need to reset ourselves to potentially be run again.
		self.reset();

		// Return that we succeeded
		Status::Succeeded
	}

	fn reset(&mut self)
	{
		// Reset our status and put our counter back to the first node
		self.status = Status::Running;
		self.next_tick_index = 0;
	}

	fn status(&self) -> Status
	{
		self.status
	}
}
