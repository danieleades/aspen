//! Nodes that have children and tick them in a sequential order as long as they
//! fail.
//!
//! NOTE: There is no Selector* node, since the choice of not having the nodes
//! automatically reset causes a normal Selector node to have the same behavior
//! as a Selector*.
use std::sync::Arc;
use node::Node;
use status::Status;

/// Implements a Selector node
///
/// This node will tick all of its children in order until one of them returns
/// either `Status::Running` or `Status::Success`. If none do, this node fails.
pub struct Selector<T: Send + Sync + 'static>
{
	/// Vector containing the children of this node
	children: Vec<Box<Node<T>>>,
}
impl<T: Send + Sync + 'static> Selector<T>
{
	/// Creates a new Selector node from a vector of Nodes
	pub fn new(children: Vec<Box<Node<T>>>) -> Selector<T>
	{
		Selector {
			children: children,
		}
	}
}
impl<T: Send + Sync + 'static> Node<T> for Selector<T>
{
	fn tick(&mut self, world: &Arc<T>) -> Status
	{
		// Tick the children in order
		for ptr in self.children.iter_mut() {
			let child_status = (*ptr).tick(world);
			if child_status != Status::Failed {
				return child_status;
			}
		}

		// All children failed
		Status::Failed
	}

	fn reset(&mut self)
	{
		// Reset all of our children
		for ptr in self.children.iter_mut() {
			(*ptr).reset();
		}
	}

	fn status(&self) -> Status
	{
		for ptr in self.children.iter() {
			let child_status = (*ptr).status();
			if child_status != Status::Failed {
				return child_status;
			}
		}

		// All children failed
		Status::Failed
	}
}
