//! Nodes that have a constant behavior.
use crate::{
    node::{Node, Tickable},
    status::Status,
};

/// Implements a node that always returns that it has failed.
///
/// This node potentially takes a child node. If it does, then it will tick that
/// node until it is completed, disregard the child's status, and return that it
/// failed. If it does not have a child node, it will simply fail on every tick.
///
/// # State
///
/// **Initialized:** Before being ticked after either being created or reset.
///
/// **Running:** While child is running. If no child, then never.
///
/// **Succeeded:** Never.
///
/// **Failed:** After child finishes. If no child, always.
///
/// # Children
///
/// One optional child. The child will be reset every time this node is reset.
///
/// # Examples
///
/// An `AlwaysFail` node always fails when it has no child:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// # use aspen::node::Tickable;
/// let mut node = AlwaysFail::new();
/// assert_eq!(node.tick(&mut ()), Status::Failed);
/// ```
///
/// If the child is considered running, so is this node:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// # use aspen::node::Tickable;
/// let mut node = AlwaysFail::with_child(AlwaysRunning::new());
/// assert_eq!(node.tick(&mut ()), Status::Running);
/// ```
///
/// If the child is done running, its status is disregarded:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// # use aspen::node::Tickable;
/// let mut node = AlwaysFail::with_child(AlwaysSucceed::new());
/// assert_eq!(node.tick(&mut ()), Status::Failed);
/// ```
pub struct AlwaysFail<'a, W> {
    /// Optional child node.
    child: Option<Node<'a, W>>,
}
impl<'a, W> AlwaysFail<'a, W>
where
    W: 'a,
{
    /// Construct a new AlwaysFail node.
    pub fn new() -> Node<'a, W> {
        Node::new(AlwaysFail { child: None })
    }

    /// Construct a new AlwaysFail node that has a child.
    pub fn with_child(child: Node<'a, W>) -> Node<'a, W> {
        Node::new(AlwaysFail { child: Some(child) })
    }
}
impl<'a, W> Tickable<W> for AlwaysFail<'a, W> {
    fn tick(&mut self, world: &mut W) -> Status {
        if let Some(ref mut child) = self.child {
            if !child.tick(world).is_done() {
                return Status::Running;
            }
        }

        Status::Failed
    }

    fn reset(&mut self) {
        if let Some(ref mut child) = self.child {
            child.reset();
        }
    }

    fn children(&self) -> Vec<&Node<W>> {
        if let Some(ref child) = self.child {
            vec![child]
        } else {
            Vec::new()
        }
    }

    /// Returns the string "AlwaysFail".
    fn type_name(&self) -> &'static str {
        "AlwaysFail"
    }
}

/// Convenience macro for creating AlwaysFail nodes.
///
/// # Examples
///
/// Without a child:
///
/// ```
/// # #[macro_use] extern crate aspen;
/// # use aspen::node::Node;
/// # fn main() {
/// let fail: Node<()> = AlwaysFail! {};
/// # }
/// ```
///
/// With a child:
///
/// ```
/// # #[macro_use] extern crate aspen;
/// # fn main() {
/// let fail_child = AlwaysFail! {
///     Condition!{ |a: &u32| *a < 12 }
/// };
/// # }
/// ```
#[macro_export]
macro_rules! AlwaysFail {
    ( $e:expr ) => {
        $crate::std_nodes::AlwaysFail::with_child($e)
    };
    ( ) => {
        $crate::std_nodes::AlwaysFail::new()
    };
}

/// Implements a node that always returns that it has succeeded.
///
/// This node potentially takes a child node. If it does, then it will tick that
/// node until it is completed, disregard the child's status, and return that it
/// succeeded. If it does not have a child node, it will simply succeed on
/// every tick.
///
/// # State
///
/// **Initialized:** Before being ticked after either being created or reset.
///
/// **Running:** While child is running. If no child, then never.
///
/// **Succeeded:** After child finished. If no child, always.
///
/// **Failed:** Never.
///
/// # Children
///
/// One optional child. The child will be reset every time this node is reset.
///
/// # Examples
///
/// An `AlwaysSucceed` node always succeeds when it has no child:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// # use aspen::node::Tickable;
/// let mut node = AlwaysSucceed::new();
/// assert_eq!(node.tick(&mut ()), Status::Succeeded);
/// ```
///
/// If the child is considered running, so is this node:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// # use aspen::node::Tickable;
/// let mut node = AlwaysSucceed::with_child(AlwaysRunning::new());
/// assert_eq!(node.tick(&mut ()), Status::Running);
/// ```
///
/// If the child is done running, its status is disregarded:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// # use aspen::node::Tickable;
/// let mut node = AlwaysSucceed::with_child(AlwaysFail::new());
/// assert_eq!(node.tick(&mut ()), Status::Succeeded);
/// ```
pub struct AlwaysSucceed<'a, W> {
    /// Optional child node.
    child: Option<Node<'a, W>>,
}
impl<'a, W> AlwaysSucceed<'a, W>
where
    W: 'a,
{
    /// Construct a new [`AlwaysSucceed`] node.
    pub fn new() -> Node<'a, W> {
        Node::new(AlwaysSucceed { child: None })
    }

    /// Construct a new [`AlwaysSucceed`] node with a child.
    pub fn with_child(child: Node<'a, W>) -> Node<'a, W> {
        Node::new(AlwaysSucceed { child: Some(child) })
    }
}
impl<'a, W> Tickable<W> for AlwaysSucceed<'a, W> {
    fn tick(&mut self, world: &mut W) -> Status {
        if let Some(ref mut child) = self.child {
            if !child.tick(world).is_done() {
                return Status::Running;
            }
        }

        Status::Succeeded
    }

    fn children(&self) -> Vec<&Node<W>> {
        if let Some(ref child) = self.child {
            vec![child]
        } else {
            Vec::new()
        }
    }

    fn reset(&mut self) {
        if let Some(ref mut child) = self.child {
            child.reset();
        }
    }

    /// Returns the string "AlwaysSucceed".
    fn type_name(&self) -> &'static str {
        "AlwaysSucceed"
    }
}

/// Convenience macro for creating AlwaysSucceed nodes.
///
/// # Examples
///
/// Without a child:
///
/// ```
/// # #[macro_use] extern crate aspen;
/// # use aspen::node::Node;
/// # fn main() {
/// let succeed: Node<()> = AlwaysSucceed! {};
/// # }
/// ```
///
/// With a child:
///
/// ```
/// # #[macro_use] extern crate aspen;
/// # fn main() {
/// let succeed_child = AlwaysSucceed! {
///     Condition!{ |a: &u32| *a < 12 }
/// };
/// # }
/// ```
#[macro_export]
macro_rules! AlwaysSucceed {
    ( $e:expr ) => {
        $crate::std_nodes::AlwaysSucceed::with_child($e)
    };
    ( ) => {
        $crate::std_nodes::AlwaysSucceed::new()
    };
}

/// Implements a node that always returns that it is currently running.
///
/// # State
///
/// **Initialized:** Before being ticked after either being created or reset.
///
/// **Running:** Always.
///
/// **Succeeded:** Never.
///
/// **Failed:** Never.
///
/// # Children
///
/// None.
///
/// # Examples
///
/// An `AlwaysRunning` node is always running:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// # use aspen::node::Tickable;
/// let mut node = AlwaysRunning::new();
/// assert_eq!(node.tick(&mut ()), Status::Running);
/// ```
pub struct AlwaysRunning;
impl AlwaysRunning {
    /// Construct a new AlwaysRunning node.
    pub fn new<W>() -> Node<'static, W> {
        Node::new(AlwaysRunning {})
    }
}
impl<W> Tickable<W> for AlwaysRunning {
    fn tick(&mut self, _: &mut W) -> Status {
        Status::Running
    }

    fn reset(&mut self) {
        // No-op
    }

    /// Returns the string "AlwaysRunning".
    fn type_name(&self) -> &'static str {
        "AlwaysRunning"
    }
}

/// Convenience macro for creating AlwaysRunning nodes.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate aspen;
/// # use aspen::node::Node;
/// # fn main() {
/// let running: Node<()> = AlwaysRunning! {};
/// # }
/// ```
#[macro_export]
macro_rules! AlwaysRunning {
    ( ) => {
        $crate::std_nodes::AlwaysRunning::new()
    };
}

#[cfg(test)]
mod tests {
    use crate::{
        node::Tickable,
        status::Status,
        std_nodes::{AlwaysFail, AlwaysRunning, AlwaysSucceed, YesTick},
    };

    #[test]
    fn always_fail() {
        assert_eq!(AlwaysFail::new().tick(&mut ()), Status::Failed);
    }

    #[test]
    fn always_fail_child() {
        let mut succeed = AlwaysFail::with_child(YesTick::new(Status::Succeeded));
        let succeed_res = succeed.tick(&mut ());
        drop(succeed);
        assert_eq!(succeed_res, Status::Failed);

        let mut run = AlwaysFail::with_child(YesTick::new(Status::Running));
        let run_res = run.tick(&mut ());
        drop(run);
        assert_eq!(run_res, Status::Running);

        let mut fail = AlwaysFail::with_child(YesTick::new(Status::Failed));
        let fail_res = fail.tick(&mut ());
        drop(fail);
        assert_eq!(fail_res, Status::Failed);
    }

    #[test]
    fn always_succeed() {
        assert_eq!(AlwaysSucceed::new().tick(&mut ()), Status::Succeeded);
    }

    #[test]
    fn always_succeed_child() {
        let mut succeed = AlwaysSucceed::with_child(YesTick::new(Status::Succeeded));
        let succeed_res = succeed.tick(&mut ());
        drop(succeed);
        assert_eq!(succeed_res, Status::Succeeded);

        let mut run = AlwaysSucceed::with_child(YesTick::new(Status::Running));
        let run_res = run.tick(&mut ());
        drop(run);
        assert_eq!(run_res, Status::Running);

        let mut fail = AlwaysSucceed::with_child(YesTick::new(Status::Failed));
        let fail_res = fail.tick(&mut ());
        drop(fail);
        assert_eq!(fail_res, Status::Succeeded);
    }

    #[test]
    fn always_running() {
        assert_eq!(AlwaysRunning::new().tick(&mut ()), Status::Running);
    }
}
