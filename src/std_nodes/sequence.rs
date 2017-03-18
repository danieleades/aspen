//! Nodes that have children and tick them in a sequential order as long as they
//! succeed.
//!
//! NOTE: There is no Sequence* node, since the choice of not having the nodes
//! automatically reset causes a normal Sequence node to have the same behavior
//! as a Sequence*.
use std::sync::Arc;
use node::{Node, Internals, IdType};
use status::Status;

/// Implements a Sequence node
///
/// This node will tick all of its children in order until one of them returns
/// either `Status::Running` or `Status::Failed`. If none do, this node succeeds.
pub struct Sequence<T: Send + Sync + 'static>
{
	/// Vector containing the children of this node
	children: Vec<Node<T>>,
}
impl<T: Send + Sync + 'static> Sequence<T>
{
	/// Creates a new Sequence node from a vector of Nodes
	pub fn new(children: Vec<Box<Node<T>>>) -> Node<T>
	{
		let internals = Sequence { children: children };
		Node::new(internals)
	}
}
impl<T: Send + Sync + 'static> Internals<T> for Sequence<T>
{
	fn tick(&mut self, world: &Arc<T>) -> Status
	{
		// Tick the children in order
		for child in self.children.iter_mut() {
			// First, tick the current child to get its status
			let child_status = child.tick(world);

			// Then decide if we're done ticking based on our children
			if child_status != Status::Succeeded {
				return child_status;
			}
		}

		// All children succeeded
		Status::Succeeded
	}

	fn reset(&mut self)
	{
		// Reset all of our children
		for child in self.children.iter_mut() {
			child.reset();
		}
	}

	fn children(&self) -> Vec<&Node<T>>
	{
		self.children.iter().collect()
	}

	fn children_ids(&self) -> Vec<IdType>
	{
		self.children.iter().map(|c| c.id()).collect()
	}

	fn type_name() -> &str
	{
		"Sequence"
	}
}

#[cfg(test)]
mod test
{
	use std::sync::Arc;
	use std::sync::atomic::AtomicBool;
	use node::Node;
	use status::Status;
	use std_nodes::*;

	#[test]
	fn check_running()
	{
		// Use an atomic as the world (doesn't actually get used)
		let world = Arc::new(AtomicBool::new(true));

		// Set up the nodes
		let children = vec![YesTick::new(Status::Succeeded),
		                    YesTick::new(Status::Running),
		                    NoTick::new()];

		// Add them to a sequence node
		let mut seq = Sequence::new(children);

		// Tick the sequence
		let status = seq.tick(&world);

		// Drop the sequence so the nodes can do their own checks
		drop(seq);

		// Make sure we got the expected value
		assert_eq!(status, Status::Running);
	}

	#[test]
	fn check_success()
	{
		// Use an atomic as the world (doesn't actually get used)
		let world = Arc::new(AtomicBool::new(true));

		// Set up the nodes
		let children = vec![YesTick::new(Status::Succeeded),
		                    YesTick::new(Status::Succeeded)];

		// Add them to a sequence node
		let mut seq = Sequence::new(children);

		// Tick the sequence
		let status = seq.tick(&world);

		// Drop the sequence so the nodes can do their own checks
		drop(seq);

		// Make sure we got the expected value
		assert_eq!(status, Status::Succeeded);
	}

	#[test]
	fn check_fail()
	{
		// Use an atomic as the world (doesn't actually get used)
		let world = Arc::new(AtomicBool::new(true));

		// Set up the nodes
		let children = vec![YesTick::new(Status::Succeeded),
		                    YesTick::new(Status::Failed),
		                    NoTick::new()];

		// Add them to a sequence node
		let mut seq = Sequence::new(children);

		// Tick the sequence
		let status = seq.tick(&world);

		// Drop the sequence so the nodes can do their own checks
		drop(seq);

		// Make sure we got the expected value
		assert_eq!(status, Status::Failed);
	}
}
