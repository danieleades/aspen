//! Nodes that have children and tick them in a sequential order as long as they succeed.
use crate::node::{Node, Tickable};
use crate::Status;

/// A node that will tick its children in order as long as they succeed.
///
/// This node will tick all of its children in order until one of them returns
/// either `Status::Running` or `Status::Failed`. If none do, this node succeeds.
///
/// The difference between this node and the normal `Sequence` is that this
/// node will always begin ticking from its first child, where as the normal
/// version will resume ticking with the node that previously returned that it
/// was running. This makes the active version better for things that must be
/// checked each tick (e.g., if motors are too hot) and the normal version better
/// for completing series of actions.
///
/// Due to the reticking, some nodes that succeeded on previous ticks may fail
/// on later ticks.
///
/// This node is equivalent to an "and" statement.
///
/// # State
///
/// **Initialized:** Before being ticked after being created or reset.
///
/// **Running:** The latest ticked child node return that it was running.
///
/// **Succeeded:** All child nodes succeeded.
///
/// **Failed:** All child nodes failed.
///
/// # Children
///
/// Any number of children. A child node will be ticked every time this node is
/// ticked as long as all the sibling nodes to the left succeeded.
///
/// Note that, if a node is running and a sibling to the left returned either
/// failure or running, the child node will be reset. Additionally, the children
/// will be reset each time the parent is.
///
/// # Examples
///
/// A node that returns success:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// # use aspen::node::Tickable;
/// let mut node = ActiveSequence::new()
/// 	.with_child(AlwaysSucceed::new())
/// 	.with_child(AlwaysSucceed::new())
/// 	.with_child(AlwaysSucceed::new());
///
/// assert_eq!(node.tick(&mut ()), Status::Succeeded);
/// ```
///
/// A node that returns it is running:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// # use aspen::node::Tickable;
/// let mut node = ActiveSequence::new()
///     .with_child(AlwaysSucceed::new())
///     .with_child(AlwaysRunning::new())
///     .with_child(AlwaysFail::new());
///
/// assert_eq!(node.tick(&mut ()), Status::Running);
/// ```
///
/// A node that returns it failed:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// # use aspen::node::Tickable;
/// let mut node = ActiveSequence::new()
///     .with_child(AlwaysSucceed::new())
///     .with_child(AlwaysSucceed::new())
///     .with_child(AlwaysFail::new());
///
/// assert_eq!(node.tick(&mut ()), Status::Failed);
/// ```
pub struct ActiveSequence<'a, W> {
    /// Vector containing the children of this node.
    children: Vec<Node<'a, W>>,
}
impl<'a, W> ActiveSequence<'a, W>
where
    W: 'a,
{
    /// Creates a new `ActiveSequence` node from a vector of Nodes.
    pub fn new() -> Self {
        ActiveSequence {
            children: Vec::new(),
        }
    }

    pub fn with_child<T>(mut self, child: T) -> Self
    where
        T: Tickable<W> + 'a,
    {
        self.children.push(child.into_node());
        self
    }

    pub fn with_children<T>(mut self, children: Vec<T>) -> Self
    where
        T: Tickable<W> + 'a,
    {
        self.children = children.into_iter().map(|x| x.into_node()).collect();
        self
    }
}
impl<'a, W> Tickable<W> for ActiveSequence<'a, W> {
    fn tick(&mut self, world: &mut W) -> Status {
        // Tick all of our children as long as they succeed
        let mut ret_status = Status::Succeeded;
        for child in self.children.iter_mut() {
            if ret_status == Status::Succeeded {
                ret_status = child.tick(world);
            } else {
                child.reset();
            }
        }

        // Return whatever result we found
        ret_status
    }

    fn reset(&mut self) {
        // Reset all of our children
        for child in self.children.iter_mut() {
            child.reset();
        }
    }

    fn children(&self) -> Vec<&Node<W>> {
        self.children.iter().collect()
    }

    /// Returns the string "ActiveSequence".
    fn type_name(&self) -> &'static str {
        "ActiveSequence"
    }
}

/// Convenience macro for creating ActiveSequence nodes.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate aspen;
/// # fn main() {
/// let active_sequence = ActiveSequence!{
///     Condition!{ |&(a, _): &(u32, u32)| a < 12 },
///     Condition!{ |&(_, b)| b == 9 },
///     Condition!{ |&(a, b)| b < a }
/// };
/// # }
/// ```
#[macro_export]
macro_rules! ActiveSequence
{
	( $( $e:expr ),* ) => {
		$crate::std_nodes::ActiveSequence::new().with_children(vec![$( $e ),*])
	};
}

/// A node that will tick its children in order as long as they succeed.
///
/// This node will tick all of its children in order until one of them returns
/// either `Status::Running` or `Status::Failed`. If none do, this node succeeds.
///
/// The difference between this node and an `ActiveSequence` is that this node
/// will resume ticking at the last running node whereas the active version will
/// always restart ticking from the beginning. That makes the active sequence
/// good for things that always need to be rechecked and this version good for
/// completing actions. Once a node is ticked to completion, this version will
/// *not* revisit it.
///
/// This node is equivalent to an "and" statement.
///
/// # State
///
/// **Initialized:** Before being ticked after being created or reset.
///
/// **Running:** The latest ticked child node return that it was running.
///
/// **Succeeded:** All child nodes succeeded.
///
/// **Failed:** All child nodes failed.
///
/// # Children
///
/// Any number of children. A child node will only be ticked if all the nodes
/// to the left succeeded and this node has not yet completed.
///
/// Unlike the active version, children nodes will only be reset when this node
/// is reset.
///
/// # Examples
///
/// A node that returns success:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// # use aspen::node::Tickable;
/// let mut node = Sequence::new(vec![
///     AlwaysSucceed::new(),
///     AlwaysSucceed::new(),
///     AlwaysSucceed::new()
/// ]);
/// assert_eq!(node.tick(&mut ()), Status::Succeeded);
/// ```
///
/// A node that returns it is running:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// # use aspen::node::Tickable;
/// let mut node = Sequence::new(vec![
///     AlwaysSucceed::new(),
///     AlwaysRunning::new(),
///     AlwaysFail::new()
/// ]);
/// assert_eq!(node.tick(&mut ()), Status::Running);
/// ```
///
/// A node that returns it failed:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// # use aspen::node::Tickable;
/// let mut node = Sequence::new(vec![
///     AlwaysSucceed::new(),
///     AlwaysSucceed::new(),
///     AlwaysFail::new()
/// ]);
/// assert_eq!(node.tick(&mut ()), Status::Failed);
/// ```
pub struct Sequence<'a, W> {
    /// Vector containing the children of this node.
    children: Vec<Node<'a, W>>,
    next_child: usize,
}
impl<'a, W> Sequence<'a, W>
where
    W: 'a,
{
    /// Creates a new `Sequence` node from a vector of Nodes.
    pub fn new(children: Vec<Node<'a, W>>) -> Node<'a, W> {
        let internals = Sequence {
            children: children,
            next_child: 0,
        };
        Node::new(internals)
    }
}
impl<'a, W> Tickable<W> for Sequence<'a, W> {
    fn tick(&mut self, world: &mut W) -> Status {
        // Tick the children as long as they keep failing
        let mut ret_status = Status::Succeeded;
        while self.next_child < self.children.len() && ret_status == Status::Succeeded {
            ret_status = self.children[self.next_child].tick(world);

            if ret_status.is_done() {
                self.next_child += 1;
            }
        }

        return ret_status;
    }

    fn reset(&mut self) {
        // Reset all of our children
        for child in self.children.iter_mut() {
            child.reset();
        }

        self.next_child = 0;
    }

    fn children(&self) -> Vec<&Node<W>> {
        self.children.iter().collect()
    }

    /// Returns the string "Sequence".
    fn type_name(&self) -> &'static str {
        "Sequence"
    }
}

/// Convenience macro for creating Selector nodes.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate aspen;
/// # fn main() {
/// let selector = Selector!{
///     Condition!{ |&(a, _): &(u32, u32)| a < 12 },
///     Condition!{ |&(_, b)| b == 9 },
///     Condition!{ |&(a, b)| b < a }
/// };
/// # }
/// ```
#[macro_export]
macro_rules! Sequence
{
	( $( $e:expr ),* ) => {
		$crate::std_nodes::Sequence::new(vec![$( $e ),*])
	};
}

#[cfg(test)]
mod tests {
    use crate::node::Tickable;
    use crate::std_nodes::*;
    use crate::Status;

    #[test]
    fn check_running() {
        // Set up the nodes
        let children = vec![
            YesTick::new(Status::Succeeded),
            YesTick::new(Status::Running),
            NoTick::new(),
        ];

        // Add them to a sequence node
        let mut seq = Sequence::new(children);

        // Tick the sequence
        let status = seq.tick(&mut ());

        // Drop the sequence so the nodes can do their own checks
        drop(seq);

        // Make sure we got the expected value
        assert_eq!(status, Status::Running);
    }

    #[test]
    fn check_success() {
        // Set up the nodes
        let children = vec![
            YesTick::new(Status::Succeeded),
            YesTick::new(Status::Succeeded),
        ];

        // Add them to a sequence node
        let mut seq = Sequence::new(children);

        // Tick the sequence
        let status = seq.tick(&mut ());

        // Drop the sequence so the nodes can do their own checks
        drop(seq);

        // Make sure we got the expected value
        assert_eq!(status, Status::Succeeded);
    }

    #[test]
    fn check_fail() {
        // Set up the nodes
        let children = vec![
            YesTick::new(Status::Succeeded),
            YesTick::new(Status::Failed),
            NoTick::new(),
        ];

        // Add them to a sequence node
        let mut seq = Sequence::new(children);

        // Tick the sequence
        let status = seq.tick(&mut ());

        // Drop the sequence so the nodes can do their own checks
        drop(seq);

        // Make sure we got the expected value
        assert_eq!(status, Status::Failed);
    }

    #[test]
    fn check_active_running() {
        // Set up the nodes
        let children = vec![
            YesTick::new(Status::Succeeded),
            YesTick::new(Status::Running),
            NoTick::new(),
        ];

        // Add them to a sequence node
        let mut seq = ActiveSequence::new().with_children(children);

        // Tick the sequence
        let status = seq.tick(&mut ());

        // Drop the sequence so the nodes can do their own checks
        drop(seq);

        // Make sure we got the expected value
        assert_eq!(status, Status::Running);
    }

    #[test]
    fn check_active_success() {
        // Set up the nodes
        let children = vec![
            YesTick::new(Status::Succeeded),
            YesTick::new(Status::Succeeded),
        ];

        // Add them to a sequence node
        let mut seq = ActiveSequence::new().with_children(children);

        // Tick the sequence
        let status = seq.tick(&mut ());

        // Drop the sequence so the nodes can do their own checks
        drop(seq);

        // Make sure we got the expected value
        assert_eq!(status, Status::Succeeded);
    }

    #[test]
    fn check_active_fail() {
        // Set up the nodes
        let children = vec![
            YesTick::new(Status::Succeeded),
            YesTick::new(Status::Failed),
            NoTick::new(),
        ];

        // Add them to a sequence node
        let mut seq = ActiveSequence::new().with_children(children);

        // Tick the sequence
        let status = seq.tick(&mut ());

        // Drop the sequence so the nodes can do their own checks
        drop(seq);

        // Make sure we got the expected value
        assert_eq!(status, Status::Failed);
    }
}
