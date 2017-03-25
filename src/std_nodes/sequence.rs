//! Nodes that have children and tick them in a sequential order as long as they
//! succeed.
//!
//! NOTE: There is no Sequence* node, since the choice of not having the nodes
//! automatically reset causes a normal Sequence node to have the same behavior
//! as a Sequence*.
use node::{Node, Internals, IdType};
use status::Status;

/// Implements a node that will tick its children in order as long as they succeed
///
/// This node will tick all of its children in order until one of them returns
/// either `Status::Running` or `Status::Failed`. If none do, this node succeeds.
/// This is roughly equivalent to an "and" statement.
pub struct Sequence
{
	/// Vector containing the children of this node
	children: Vec<Node>,
}
impl Sequence
{
	/// Creates a new Sequence node from a vector of Nodes
	pub fn new(children: Vec<Node>) -> Node
	{
		let internals = Sequence { children: children };
		Node::new(internals)
	}
}
impl Internals for Sequence
{
	/// Ticks all of this node's children in order. If a child returns
	/// something other than `Status::Succeeded`, this node stops ticking its
	/// children and returns the child's status. If all nodes succeed, so does
	/// this node.
	fn tick(&mut self) -> Status
	{
		// Tick the children in order
		for child in self.children.iter_mut() {
			// First, tick the current child to get its status
			let child_status = child.tick();

			// Then decide if we're done ticking based on our children
			if child_status != Status::Succeeded {
				return child_status;
			}
		}

		// All children succeeded
		Status::Succeeded
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
	fn children(&self) -> Vec<&Node>
	{
		self.children.iter().collect()
	}

	fn children_ids(&self) -> Vec<IdType>
	{
		self.children.iter().map(|c| c.id()).collect()
	}

	/// Returns the string "Sequence"
	fn type_name(&self) -> &'static str
	{
		"Sequence"
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
		let children = vec![YesTick::new(Status::Succeeded),
		                    YesTick::new(Status::Running),
		                    NoTick::new()];

		// Add them to a sequence node
		let mut seq = Sequence::new(children);

		// Tick the sequence
		let status = seq.tick();

		// Drop the sequence so the nodes can do their own checks
		drop(seq);

		// Make sure we got the expected value
		assert_eq!(status, Status::Running);
	}

	#[test]
	fn check_success()
	{
		// Set up the nodes
		let children = vec![YesTick::new(Status::Succeeded),
		                    YesTick::new(Status::Succeeded)];

		// Add them to a sequence node
		let mut seq = Sequence::new(children);

		// Tick the sequence
		let status = seq.tick();

		// Drop the sequence so the nodes can do their own checks
		drop(seq);

		// Make sure we got the expected value
		assert_eq!(status, Status::Succeeded);
	}

	#[test]
	fn check_fail()
	{
		// Set up the nodes
		let children = vec![YesTick::new(Status::Succeeded),
		                    YesTick::new(Status::Failed),
		                    NoTick::new()];

		// Add them to a sequence node
		let mut seq = Sequence::new(children);

		// Tick the sequence
		let status = seq.tick();

		// Drop the sequence so the nodes can do their own checks
		drop(seq);

		// Make sure we got the expected value
		assert_eq!(status, Status::Failed);
	}
}
