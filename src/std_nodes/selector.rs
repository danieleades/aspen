//! Nodes that have children and tick them in a sequential order as long as they
//! fail.
//!
//! NOTE: There is no Selector* node, since the choice of not having the nodes
//! automatically reset causes a normal Selector node to have the same behavior
//! as a Selector*.
use node::{Node, Internals, IdType};
use status::Status;

/// Implements a node that ticks its children in order as long as they fail
///
/// This node will tick all of its children in order until one of them returns
/// either `Status::Running` or `Status::Success`. If none do, this node fails.
/// This is roughly equivalent to an "or" statement.
pub struct Selector
{
	/// Vector containing the children of this node
	children: Vec<Node>,
}
impl Selector
{
	/// Creates a new Selector node from a vector of Nodes
	pub fn new(children: Vec<Node>) -> Node
	{
		let internals = Selector { children: children };
		Node::new(internals)
	}
}
impl Internals for Selector
{
	/// Ticks all of the children nodes in order. If a child returns something
	/// other than than `Status::Failed`, this node will stop ticking its
	/// children and return the child's status. If all children fail, so does
	/// this node.
	fn tick(&mut self) -> Status
	{
		// Tick the children in order
		for child in self.children.iter_mut() {
			let child_status = child.tick();
			if child_status != Status::Failed {
				return child_status;
			}
		}

		// All children failed
		Status::Failed
	}

	/// Resets this node and all of its children
	fn reset(&mut self)
	{
		// Reset all of our children
		for child in self.children.iter_mut() {
			child.reset();
		}
	}

	/// Returns a vector containing references to all of this node's children
	fn children(&self) -> Option<Vec<&Node>>
	{
		Some(self.children.iter().collect())
	}

	fn children_ids(&self) -> Option<Vec<IdType>>
	{
		Some(self.children.iter().map(|c| c.id()).collect())
	}

	/// Returns the string "Selector"
	fn type_name(&self) -> &'static str
	{
		"Selector"
	}
}

#[cfg(test)]
mod test
{
	use status::Status;
	use std_nodes::*;

	#[test]
	fn check_running()
	{
		// Set up the nodes
		let children = vec![YesTick::new(Status::Failed),
		                    YesTick::new(Status::Running),
		                    NoTick::new()];

		// Add them to a seluence node
		let mut sel = Selector::new(children);

		// Tick the seluence
		let status = sel.tick();

		// Drop the selector so the nodes can do their own checks
		drop(sel);

		// Make sure we got the expected value
		assert_eq!(status, Status::Running);
	}

	#[test]
	fn check_success()
	{
		// Set up the nodes
		let children = vec![YesTick::new(Status::Failed),
		                    YesTick::new(Status::Succeeded),
		                    NoTick::new()];

		// Add them to a seluence node
		let mut sel = Selector::new(children);

		// Tick the seluence
		let status = sel.tick();

		// Drop the selector so the nodes can do their own checks
		drop(sel);

		// Make sure we got the expected value
		assert_eq!(status, Status::Succeeded);
	}

	#[test]
	fn check_fail()
	{
		// Set up the nodes
		let children = vec![YesTick::new(Status::Failed),
		                    YesTick::new(Status::Failed)];

		// Add them to a selector node
		let mut sel = Selector::new(children);

		// Tick the seluence
		let status = sel.tick();

		// Drop the selector so the nodes can do their own checks
		drop(sel);

		// Make sure we got the expected value
		assert_eq!(status, Status::Failed);
	}
}
