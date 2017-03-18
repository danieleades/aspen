//! Nodes that have children and tick them in a sequential order as long as they
//! fail.
//!
//! NOTE: There is no Selector* node, since the choice of not having the nodes
//! automatically reset causes a normal Selector node to have the same behavior
//! as a Selector*.
use std::sync::Arc;
use node::{Node, Internals, IdType};
use status::Status;

/// Implements a Selector node
///
/// This node will tick all of its children in order until one of them returns
/// either `Status::Running` or `Status::Success`. If none do, this node fails.
pub struct Selector<T: Send + Sync + 'static>
{
	/// Vector containing the children of this node
	children: Vec<Node<T>>,
}
impl<T: Send + Sync + 'static> Selector<T>
{
	/// Creates a new Selector node from a vector of Nodes
	pub fn new(children: Vec<Node<T>>) -> Node<T>
	{
		let internals = Selector { children: children };
		Node::new(internals)
	}
}
impl<T: Send + Sync + 'static> Internals<T> for Selector<T>
{
	fn tick(&mut self, world: &Arc<T>) -> Status
	{
		// Tick the children in order
		for child in self.children.iter_mut() {
			let child_status = child.tick(world);
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
		"Selector"
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
		let children = vec![YesTick::new(Status::Failed),
		                    YesTick::new(Status::Running),
		                    NoTick::new()];

		// Add them to a seluence node
		let mut sel = Selector::new(children);

		// Tick the seluence
		let status = sel.tick(&world);

		// Drop the selector so the nodes can do their own checks
		drop(sel);

		// Make sure we got the expected value
		assert_eq!(status, Status::Running);
	}

	#[test]
	fn check_success()
	{
		// Use an atomic as the world (doesn't actually get used)
		let world = Arc::new(AtomicBool::new(true));

		// Set up the nodes
		let children = vec![YesTick::new(Status::Failed),
		                    YesTick::new(Status::Succeeded),
		                    NoTick::new()];

		// Add them to a seluence node
		let mut sel = Selector::new(children);

		// Tick the seluence
		let status = sel.tick(&world);

		// Drop the selector so the nodes can do their own checks
		drop(sel);

		// Make sure we got the expected value
		assert_eq!(status, Status::Succeeded);
	}

	#[test]
	fn check_fail()
	{
		// Use an atomic as the world (doesn't actually get used)
		let world = Arc::new(AtomicBool::new(true));

		// Set up the nodes
		let children = vec![YesTick::new(Status::Failed),
		                    YesTick::new(Status::Failed)];

		// Add them to a selector node
		let mut sel = Selector::new(children);

		// Tick the seluence
		let status = sel.tick(&world);

		// Drop the selector so the nodes can do their own checks
		drop(sel);

		// Make sure we got the expected value
		assert_eq!(status, Status::Failed);
	}
}
