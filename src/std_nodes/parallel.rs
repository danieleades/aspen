//! Nodes that tick their children in parallel
use node::{Node, Internals};
use status::Status;

/// A node that handles "concurrent" behavior.
///
/// Every tick, this node will tick all of its children that have not been run
/// to completion. Success or failure of this node is determined by how many of
/// its children are in a successful or failed state. If the specified number
/// of children have succeeded, this node succeeds. If it is impossible for the
/// remaining children to bring the success count to the required threshold then
/// the node fails. Otherwise it is considered running.
///
/// Note that a threshold of zero means this node always succeeds on the first
/// tick and a threshold greater than the number of children means this node
/// always fails on the first tick.
///
/// It is also important to note that this node can cause child `Action` nodes
/// to actually run in parallel.
///
/// # State
///
/// **Initialized:** Before being ticked after either being created or reset.
///
/// **Running:** As long as the successful child count is below the threshold
/// and the running children could potentially make the successful count cross
/// the threshold.
///
/// **Succeeded:** The count of successful children is greater than the threshold.
///
/// **Failed:** The sum of the successful children and the running children is
/// smaller than the threshold.
///
/// # Children
///
/// Any number. The children will be reset when this node is reset but may not
/// necessarily ticked when this node is, depending on their current status.
///
/// There is a possibility that some children may not be ticked to completion
/// based on when the `Parallel` node crosses its success or failure threshold.
///
/// # Examples
///
/// A node that has enough successful children:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// let threshold = 3;
/// let mut node = Parallel::new(vec![
///     AlwaysSucceed::new(),
///     AlwaysSucceed::new(),
///     AlwaysSucceed::new(),
///     AlwaysRunning::new(),
///     AlwaysFail::new()
/// ], threshold);
///
/// assert_eq!(node.tick(), Status::Succeeded);
/// ```
///
/// A node that could either succeed or fail, so it is still running:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// let threshold = 3;
/// let mut node = Parallel::new(vec![
///     AlwaysSucceed::new(),
///     AlwaysSucceed::new(),
///     AlwaysRunning::new(),
///     AlwaysRunning::new(),
///     AlwaysFail::new()
/// ], threshold);
///
/// assert_eq!(node.tick(), Status::Running);
/// ```
///
/// A node that could not possibly succeed, so it fails:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// let threshold = 4;
/// let mut node = Parallel::new(vec![
///     AlwaysSucceed::new(),
///     AlwaysSucceed::new(),
///     AlwaysRunning::new(),
///     AlwaysFail::new(),
///     AlwaysFail::new()
/// ], threshold);
///
/// assert_eq!(node.tick(), Status::Failed);
/// ```
pub struct Parallel<'a>
{
	/// Child nodes.
	children: Vec<Node<'a>>,

	/// Number of child nodes required to succeed.
	required_successes: usize,
}
impl<'a> Parallel<'a>
{
	/// Creates a `Parallel` node with the given children an required number of successes.
	pub fn new(children: Vec<Node<'a>>, required_successes: usize) -> Node<'a>
	{
		let internals = Parallel {
			children: children,
			required_successes: required_successes,
		};
		Node::new(internals)
	}
}
impl<'a> Internals for Parallel<'a>
{
	fn tick(&mut self) -> Status
	{
		let mut successes = 0;
		let mut failures = 0;

		// Go through all the children to determine success or failure
		for child in self.children.iter_mut() {
			// Check if this child has already completed
			let child_status = if child.status().is_done() {
				// It has, so we don't want to tick it again and accidentally
				// restart it
				child.status()
			} else { child.tick() };

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

	fn reset(&mut self)
	{
		// Reset all of our children
		for child in self.children.iter_mut() {
			child.reset();
		}
	}

	fn children(&self) -> Vec<&Node>
	{
		self.children.iter().collect()
	}

	/// Returns the string "Parallel".
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
