//! Nodes that tick their children in parallel
use crate::node::{Node, Tickable};
use crate::status::Status;

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
/// # use aspen::node::Tickable;
/// let threshold = 3;
/// let mut node = Parallel::new(threshold, vec![
///     AlwaysSucceed::new(),
///     AlwaysSucceed::new(),
///     AlwaysSucceed::new(),
///     AlwaysRunning::new(),
///     AlwaysFail::new()
/// ]);
///
/// assert_eq!(node.tick(&mut ()), Status::Succeeded);
/// ```
///
/// A node that could either succeed or fail, so it is still running:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// # use aspen::node::Tickable;
/// let threshold = 3;
/// let mut node = Parallel::new(threshold, vec![
///     AlwaysSucceed::new(),
///     AlwaysSucceed::new(),
///     AlwaysRunning::new(),
///     AlwaysRunning::new(),
///     AlwaysFail::new()
/// ]);
///
/// assert_eq!(node.tick(&mut ()), Status::Running);
/// ```
///
/// A node that could not possibly succeed, so it fails:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// # use aspen::node::Tickable;
/// let threshold = 4;
/// let mut node = Parallel::new(threshold, vec![
///     AlwaysSucceed::new(),
///     AlwaysSucceed::new(),
///     AlwaysRunning::new(),
///     AlwaysFail::new(),
///     AlwaysFail::new()
/// ]);
///
/// assert_eq!(node.tick(&mut ()), Status::Failed);
/// ```
pub struct Parallel<'a, W>
{
	/// Child nodes.
	children: Vec<Node<'a, W>>,

	/// Number of child nodes required to succeed.
	required_successes: usize,
}
impl<'a, W> Parallel<'a, W>
	where W: 'a
{
	/// Creates a `Parallel` node with the given children an required number of successes.
	pub fn new(required_successes: usize, children: Vec<Node<'a, W>>) -> Node<'a, W>
	{
		let internals = Parallel {
			children: children,
			required_successes: required_successes,
		};
		Node::new(internals)
	}
}
impl<'a, W> Tickable<W> for Parallel<'a, W>
{
	fn tick(&mut self, world: &mut W) -> Status
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
			} else { child.tick(world) };

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

	fn children(&self) -> Vec<&Node<W>>
	{
		self.children.iter().collect()
	}

	/// Returns the string "Parallel".
	fn type_name(&self) -> &'static str
	{
		"Parallel"
	}
}

/// Convenience macro for creating Parallel nodes.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate aspen;
/// # fn main() {
/// # let (a, b, c, d) = (12, 13, 11, 10);
/// let parallel = Parallel!{ 3,
///     Condition!{ |&(a, _): &(u32, u32)| a < 12 },
///     Condition!{ |&(_, b)| b == 9 },
///     Condition!{ |&(a, b)| a < b }
/// };
/// # }
/// ```
#[macro_export]
macro_rules! Parallel
{
	( $c:expr, $( $e:expr ),* ) => {
		$crate::std_nodes::Parallel::new($c, vec![$( $e ),*])
	};
}

#[cfg(test)]
mod tests
{
	use crate::status::Status;
	use crate::std_nodes::*;
	use crate::node::Tickable;

	#[test]
	fn success()
	{
		let children = vec![YesTick::new(Status::Succeeded),
		                    YesTick::new(Status::Succeeded),
		                    YesTick::new(Status::Running),
		                    YesTick::new(Status::Running),
		                    YesTick::new(Status::Failed),
		                    YesTick::new(Status::Failed)];
		let mut parallel = Parallel::new(2, children);
		let status = parallel.tick(&mut ());
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
		let mut parallel = Parallel::new(5, children);
		let status = parallel.tick(&mut ());
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
		let mut parallel = Parallel::new(3, children);
		let status = parallel.tick(&mut ());
		drop(parallel);
		assert_eq!(status, Status::Running);
	}
}
