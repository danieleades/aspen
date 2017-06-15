//! Nodes that have children and tick them in a sequential order as long as they fail.
use node::{Node, Internals};
use ::Status;

/// A node that ticks its children sequentially as long as they fail.
///
/// This node will tick all of its children in order until one of them returns
/// either `Status::Running` or `Status::Success`. If none do, this node fails.
///
/// The difference between this node and a normal `Selector` is that this node
/// begins ticking at its first child every single time: the `Selector` will only
/// tick a node to completion. That makes the active version of the selector
/// good for things like monitoring if motors are too hot (which should be
/// checked every tick) whereas the normal selector is better for simply
/// completing a sequence of actions.
///
/// Due to the reticking, some nodes that succeeded on previous ticks may fail
/// on later ticks.
///
/// This is equivalent to an "or" statement.
///
/// # State
///
/// **Initialized:** Before being ticked after being created or reset.
///
/// **Running:** The latest ticked child node returned that it was running.
///
/// **Succeeded:** At least one of the children succeeded.
///
/// **Failed:** All of the children failed.
///
/// # Children
///
/// Any number of children. A child node will be ticked every time this node is
/// ticked as long as all the sibling nodes to the left failed.
///
/// Note that, if a node is running and a sibling to the left returned either
/// success or running, the child node will be reset. Additionally, the children
/// will be reset each time the parent node is reset.
///
/// # Examples
///
/// A node that returns success:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// let mut node = ActiveSelector::new(vec![
///     AlwaysFail::new(),
///     AlwaysSucceed::new(),
///     AlwaysRunning::new()
/// ]);
/// assert_eq!(node.tick(), Status::Succeeded);
/// ```
///
/// A node that returns that it is running:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// let mut node = ActiveSelector::new(vec![
///     AlwaysFail::new(),
///     AlwaysRunning::new(),
///     AlwaysSucceed::new()
/// ]);
/// assert_eq!(node.tick(), Status::Running);
/// ```
///
/// A node that returns that it fails:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// let mut node = ActiveSelector::new(vec![
///     AlwaysFail::new(),
///     AlwaysFail::new(),
///     AlwaysFail::new()
/// ]);
/// assert_eq!(node.tick(), Status::Failed);
/// ```
pub struct ActiveSelector<'a, S>
{
	/// Vector containing the children of this node.
	children: Vec<Node<'a, S>>,
}
impl<'a, S> ActiveSelector<'a, S>
	where S: 'a
{
	/// Creates a new Selector node from a vector of Nodes.
	pub fn new(children: Vec<Node<'a, S>>) -> Node<'a, S>
	{
		let internals = ActiveSelector { children: children };
		Node::new(internals)
	}
}
impl<'a, S> Internals<S> for ActiveSelector<'a, S>
{
	fn tick(&mut self, world: &mut S) -> Status
	{
		// Tick the children in order
		let mut ret_status = Status::Failed;
		for child in self.children.iter_mut() {
			// What we want to do is tick our children until we find one that
			// is either running or successful. If we find either of those, all
			// children after that node need to be reset
			if ret_status != Status::Failed {
				child.reset()
			}
			else {
				ret_status = child.tick(world);
			}
		}

		// Return the status that we found
		ret_status
	}

	fn reset(&mut self)
	{
		// Reset all of our children
		for child in self.children.iter_mut() {
			child.reset();
		}
	}

	fn children(&self) -> Vec<&Node<S>>
	{
		self.children.iter().collect()
	}

	/// Returns the string "ActiveSelector".
	fn type_name(&self) -> &'static str
	{
		"ActiveSelector"
	}
}

/// Convenience macro for creating ActiveSelector nodes.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate aspen;
/// # fn main() {
/// # let (a, b, c, d) = (12, 13, 11, 10);
/// let active_selector = ActiveSelector!{
///     Condition!{ || a < b },
///     Condition!{ || c == d },
///     Condition!{ || d < a }
/// };
/// # }
/// ```
#[macro_export]
macro_rules! ActiveSelector
{
	( $( $e:expr ),* ) => {
		$crate::std_nodes::ActiveSelector::new(vec![$( $e ),*])
	};
}

/// A node that ticks its children sequentially as long as they fail.
///
/// This node will tick all of its children in order until one of them returns
/// either `Status::Running` or `Status::Success`. If none do, this node fails.
///
/// The difference between this node and an `ActiveSelector` is that this node
/// will resume ticking at the last running node whereas the active version
/// will always restart ticking from the beginning. That makes the active
/// selector good for things that always need to be rechecked and this version
/// good at completing actions. Once a node is ticked to completion, this
/// normal selector will *not* revisit it.
///
/// This is equivalent to an "or" statement.
///
/// # State
///
/// **Initialized:** Before being ticked after being created or reset.
///
/// **Running:** A child node returned that it was running.
///
/// **Succeeded:** At least one of the children succeeded.
///
/// **Failed:** All of the children failed.
///
/// # Children
///
/// Any number of children. A child node will only be ticked if all the nodes
/// to the left failed and this node has not yet completed.
///
/// All children nodes will be reset only when this node is reset.
///
/// # Examples
///
/// A node that returns success:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// let mut node = Selector::new(vec![
///     AlwaysFail::new(),
///     AlwaysSucceed::new(),
///     AlwaysRunning::new()
/// ]);
/// assert_eq!(node.tick(), Status::Succeeded);
/// ```
///
/// A node that returns that it is running:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// let mut node = Selector::new(vec![
///     AlwaysFail::new(),
///     AlwaysRunning::new(),
///     AlwaysSucceed::new()
/// ]);
/// assert_eq!(node.tick(), Status::Running);
/// ```
///
/// A node that returns that it fails:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// let mut node = Selector::new(vec![
///     AlwaysFail::new(),
///     AlwaysFail::new(),
///     AlwaysFail::new()
/// ]);
/// assert_eq!(node.tick(), Status::Failed);
/// ```
pub struct Selector<'a, S>
{
	/// Vector containing the children of this node.
	children: Vec<Node<'a, S>>,

	/// The next child to be ticked.
	///
	/// While it feels less Rusty, doing an index seemed cleaner than any
	/// iterator version that I could come up with.
	next_child: usize,
}
impl<'a, S> Selector<'a, S>
	where S: Clone + 'a
{
	/// Creates a new Selector node from a vector of Nodes.
	pub fn new(children: Vec<Node<'a, S>>) -> Node<'a, S>
	{
		let internals = Selector {
			children: children,
			next_child: 0,
		};
		Node::new(internals)
	}
}
impl<'a, S> Internals<S> for Selector<'a, S>
	where S: Clone
{
	fn tick(&mut self, world: &mut S) -> Status
	{
		// Tick the children as long as they keep failing
		let mut ret_status = Status::Failed;
		while self.next_child < self.children.len() && ret_status == Status::Failed {
			ret_status = self.children[self.next_child].tick(world);

			if ret_status.is_done() {
				self.next_child += 1;
			}
		}

		return ret_status;
	}

	fn reset(&mut self)
	{
		// Reset all of our children
		for child in self.children.iter_mut() {
			child.reset();
		}
	}

	fn children(&self) -> Vec<&Node<S>>
	{
		self.children.iter().collect()
	}

	/// Returns the string "Selector".
	fn type_name(&self) -> &'static str
	{
		"Selector"
	}
}

/// Convenience macro for creating Selector nodes.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate aspen;
/// # fn main() {
/// # let (a, b, c, d) = (12, 13, 11, 10);
/// let selector = Selector!{
///     Condition!{ || a < b },
///     Condition!{ || c == d },
///     Condition!{ || d < a }
/// };
/// # }
/// ```
#[macro_export]
macro_rules! Selector
{
	( $( $e:expr ),* ) => {
		$crate::std_nodes::Selector::new(vec![$( $e ),*])
	};
}

#[cfg(test)]
mod test
{
	use ::Status;
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

	#[test]
	fn check_active_running()
	{
		// Set up the nodes
		let children = vec![YesTick::new(Status::Failed),
		                    YesTick::new(Status::Running),
		                    NoTick::new()];

		// Add them to a seluence node
		let mut sel = ActiveSelector::new(children);

		// Tick the seluence
		let status = sel.tick();

		// Drop the selector so the nodes can do their own checks
		drop(sel);

		// Make sure we got the expected value
		assert_eq!(status, Status::Running);
	}

	#[test]
	fn check_active_success()
	{
		// Set up the nodes
		let children = vec![YesTick::new(Status::Failed),
		                    YesTick::new(Status::Succeeded),
		                    NoTick::new()];

		// Add them to a seluence node
		let mut sel = ActiveSelector::new(children);

		// Tick the seluence
		let status = sel.tick();

		// Drop the selector so the nodes can do their own checks
		drop(sel);

		// Make sure we got the expected value
		assert_eq!(status, Status::Succeeded);
	}

	#[test]
	fn check_active_fail()
	{
		// Set up the nodes
		let children = vec![YesTick::new(Status::Failed),
		                    YesTick::new(Status::Failed)];

		// Add them to a selector node
		let mut sel = ActiveSelector::new(children);

		// Tick the seluence
		let status = sel.tick();

		// Drop the selector so the nodes can do their own checks
		drop(sel);

		// Make sure we got the expected value
		assert_eq!(status, Status::Failed);
	}
}
