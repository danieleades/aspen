//! Nodes that tick their children in parallel
use node::{Node, Internals, IdType};
use status::Status;

/// Implements a node that will tick all of its children every time it is
/// ticked. This effectively runs all of its children in parallel.
///
/// Success or failure is determined by the count of children that succeeded.
pub struct Parallel
{
	/// Children to be ticked
	children: Vec<Node>,

	/// Number of nodes required to succeed before this one does
	required_successes: usize,
}
impl Parallel
{
	/// Creates a new Parallel node with the given children. If a number of
	/// children greater-than or equal-to `required_successes` succeed, then this
	/// node will also succeed. If it ever becomes impossible for the count of
	/// successful nodes to meet that criteria, then this node fails. Otherwise,
	/// it is considered running.
	///
	/// Note that requiring zero successes means that this node will instantly
	/// succeed and requiring more successes than it has children means that this
	/// node will instantly fail.
	pub fn new(children: Vec<Node>, required_successes: usize) -> Node
	{
		let internals = Parallel {
			children: children,
			required_successes: required_successes,
		};
		Node::new(internals)
	}
}
impl Internals for Parallel
{
	/// Ticks all of this node children. The return status is determined by the
	/// number of children that succeeded and the required number of successes.
	fn tick(&mut self) -> Status
	{
		let mut successes = 0;
		let mut failures = 0;

		// Tick every single child node
		for child in self.children.iter_mut() {
			let child_status = child.tick();

			if child_status == Status::Succeeded {
				successes += 1;
			}
			else if child_status == Status::Failed {
				failures += 1;
			}
		}

		// Return a result based on the children
		if successes >= self.required_successes {
			// Enough children succeeded
			Status::Succeeded
		} else if failures + self.required_successes > self.children.len() {
			// Too many children failed - it is impossible to succeed. I
			// suspect the overflow condition to be significantly less likely
			// than the underflow, which is why I've written the condition this
			// way.
			Status::Failed
		} else {
			// Status is still undetermined
			Status::Running
		}
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
	fn children(&self) -> Option<&Vec<Node>>
	{
		Some(&self.children)
	}

	fn children_ids(&self) -> Option<Vec<IdType>>
	{
		Some(self.children.iter().map(|c| c.id()).collect())
	}

	/// Returns the string "Parallel"
	fn type_name(&self) -> &'static str
	{
		"Parallel"
	}
}

#[cfg(test)]
mod test
{
	use status::Status;
	use std_nodes::*;

	#[test]
	fn success()
	{
		let children = vec![YesTick::new(Status::Succeeded),
		                    YesTick::new(Status::Succeeded),
		                    YesTick::new(Status::Running),
		                    YesTick::new(Status::Running),
		                    YesTick::new(Status::Failed),
		                    YesTick::new(Status::Failed)];
		let mut parallel = Parallel::new(children, 2);
		let status = parallel.tick();
		drop(parallel);
		assert_eq!(status, Status::Succeeded);
	}

	#[test]
	fn failure()
	{
		let children = vec![YesTick::new(Status::Succeeded),
		                    YesTick::new(Status::Succeeded),
		                    YesTick::new(Status::Running),
		                    YesTick::new(Status::Running),
		                    YesTick::new(Status::Failed),
		                    YesTick::new(Status::Failed)];
		let mut parallel = Parallel::new(children, 5);
		let status = parallel.tick();
		drop(parallel);
		assert_eq!(status, Status::Failed);
	}

	#[test]
	fn running()
	{
		let children = vec![YesTick::new(Status::Succeeded),
		                    YesTick::new(Status::Succeeded),
		                    YesTick::new(Status::Running),
		                    YesTick::new(Status::Running),
		                    YesTick::new(Status::Failed),
		                    YesTick::new(Status::Failed)];
		let mut parallel = Parallel::new(children, 3);
		let status = parallel.tick();
		drop(parallel);
		assert_eq!(status, Status::Running);
	}
}
