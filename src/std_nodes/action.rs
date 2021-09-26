//! Nodes that cause the execution of tasks.
use crate::{
    node::{Node, Tickable},
    status::Status,
};
use std::{
    sync::{mpsc, mpsc::TryRecvError, Arc},
    thread,
};

/// A node that manages the execution of tasks in a separate thread.
///
/// This node will launch the supplied function in a separate thread and ticks
/// will monitor the state of that thread. The return value of the function is
/// the status of the Action node.
///
/// This node should be the main way of modifying the world state. Note that,
/// despite the function being run in a separate thread, there will usually
/// only be one thread modifying the world.
///
/// Note that the supplied function will be called again the next tick if the
/// function returns either `Initialized` or `Running`.
///
/// # State
///
/// **Initialized:** Before being ticked after either being created or reset,
/// or if the function returned `Initialized`.
///
/// **Running:** While the function is being executed in the other thread or if
/// the function returned `Running`.
///
/// **Succeeded:** When the function returns `Succeeded`.
///
/// **Failed:** When the function returns `Failed`.
///
/// # Children
///
/// None.
///
/// # Examples
///
/// An action node that attempts to subtract two unsigned integers:
///
/// ```
/// # use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
/// # use std::sync::Arc;
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// # use aspen::node::Tickable;
/// const FIRST: usize = 10;
/// const SECOND: usize = 100;
/// let mut result = Arc::new(AtomicUsize::default());
///
/// let mut action = Action::new(|s: Arc<AtomicUsize>| {
///     if let Some(val) = SECOND.checked_sub(FIRST) {
///         s.store(val, Ordering::SeqCst);
///         Status::Succeeded
///     } else {
///         Status::Failed
///     }
/// });
///
/// // Run the node until it completes
/// while !action.tick(&mut result).is_done() {}
/// assert_eq!(action.status().unwrap(), Status::Succeeded);
/// assert_eq!(result.load(Ordering::SeqCst), 90);
/// ```
pub struct Action<W>
where
    W: Clone + Send + Sync + 'static,
{
    /// The task which is to be run.
    func: Arc<dyn Fn(W) -> Status + Send + Sync>,

    /// Channel on which the task will communicate.
    rx: Option<mpsc::Receiver<Status>>,
}
impl<W> Action<W>
where
    W: Clone + Send + Sync + 'static,
{
    /// Creates a new Action node that will execute the given task.
    pub fn new<F>(task: F) -> Node<'static, W>
    where
        F: Fn(W) -> Status + Send + Sync + 'static,
    {
        let internals = Action {
            func: Arc::new(task),
            rx: None,
        };

        Node::new(internals)
    }

    /// Launches a new worker thread to run the task.
    fn start_thread(&mut self, world: &W) {
        // Create our new channels
        let (tx, rx) = mpsc::sync_channel(0);

        // Then clone the function so we can move it
        let func_clone = self.func.clone();

        // Finally, boot up the thread
        let world_clone = world.clone();
        thread::spawn(move || tx.send((func_clone)(world_clone)).unwrap());

        // Store the rx for later use
        self.rx = Some(rx);
    }
}
impl<W> Tickable<W> for Action<W>
where
    W: Clone + Send + Sync + 'static,
{
    /// Ticks the Action node a single time.
    ///
    /// The first time being ticked after being reset (or initialized), it will
    /// clone `world` and use the clone as the argument for the task function,
    /// which will be run in a separate thread. Usually, this should be an
    /// `Arc`.
    fn tick(&mut self, world: &mut W) -> Status {
        let (status, reset) = if let Some(ref mut rx) = self.rx {
            match rx.try_recv() {
                Ok(Status::Running) => (Status::Running, true),
                Ok(s) => (s, false),
                Err(TryRecvError::Empty) => (Status::Running, false),
                Err(e) => panic!("Thread died before finishing {}", e),
            }
        } else {
            self.start_thread(world);
            (Status::Running, false)
        };

        if reset {
            self.rx.take();
        }

        status
    }

    /// Resets the internal state of this node.
    ///
    /// If there is a task currently running, this will block until the task is
    /// completed.
    fn reset(&mut self) {
        // I debated what to do here for a while. I could see someone wanting to detach
        // the thread due to time constraints, but it seems to me that it would be
        // better to avoid potential bugs that come from a node only looking
        // like its been fully reset.
        if let Some(ref mut rx) = self.rx {
            rx.recv().unwrap();
        }
        self.rx = None;
    }

    /// Returns the constant string "Action"
    fn type_name(&self) -> &'static str {
        "Action"
    }
}

/// Convenience macro for creating Action nodes.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate aspen;
/// # fn foo(_: ()) -> aspen::Status { aspen::Status::Succeeded }
/// # fn main() {
/// let mut action = Action! { |s| foo(s) };
/// # }
/// ```
#[macro_export]
macro_rules! Action {
    ( $e:expr ) => {
        $crate::std_nodes::Action::new($e)
    };
}

/// A node that manages the execution of tasks within the ticking thread.
///
/// This node is an alternative to a normal Action node which can be used when
/// the time required to do the task is significantly less than a single tick
/// or if it can be broken down into discrete steps. If the task takes too
/// long, or too many of these nodes are utilized, the ticking rate can be
/// affected.
///
/// # State
///
/// **Initialized:** Before being ticked after either being created or reset,
/// or if the supplied function returns `Initialized`.
///
/// **Running:** Whe the function returns `Running`.
///
/// **Succeeded:** When the function returns `Succeeded`.
///
/// **Failed:** When the function returns `Failed`.
///
/// # Children
///
/// None.
///
/// # Examples
///
/// A short action node that attempts to subtract two unsigned integers:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// # use aspen::node::Tickable;
/// let first = 10u32;
/// let second = 100u32;
/// let mut result = 0u32;
///
/// let mut action = InlineAction::new(|r| {
///     if let Some(n) = second.checked_sub(first) {
///         *r = n;
///         Status::Succeeded
///     } else {
///         Status::Failed
///     }
/// });
///
/// assert_eq!(action.tick(&mut result), Status::Succeeded);
/// assert_eq!(result, 90);
/// ```
pub struct InlineAction<'a, W> {
    /// The task which is to be run.
    func: Box<dyn FnMut(&mut W) -> Status + 'a>,
}
impl<'a, W> InlineAction<'a, W>
where
    W: 'a,
{
    /// Creates a new `ShortAction` node that will execute the given task.
    pub fn new<F>(task: F) -> Node<'a, W>
    where
        F: FnMut(&mut W) -> Status + 'a,
    {
        let internals = InlineAction {
            func: Box::new(task),
        };

        Node::new(internals)
    }
}
impl<'a, W> Tickable<W> for InlineAction<'a, W> {
    fn tick(&mut self, world: &mut W) -> Status {
        (*self.func)(world)
    }

    fn reset(&mut self) {
        // No-op
    }

    /// Returns the constant string "InlineAction"
    fn type_name(&self) -> &'static str {
        "InlineAction"
    }
}

/// Convenience macro for creating [`InlineAction`] nodes.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate aspen;
/// # use aspen::Status;
/// # fn foo(_: &mut ()) -> Status { Status::Running }
/// # fn main() {
/// let mut action = InlineAction! { |s| foo(s) };
/// # }
/// ```
#[macro_export]
macro_rules! InlineAction {
    ( $e:expr ) => {
        $crate::std_nodes::InlineAction::new($e)
    };
}

#[cfg(test)]
mod test {
    use crate::{
        node::Tickable,
        status::Status,
        std_nodes::{Action, InlineAction},
    };
    use std::{
        sync::{mpsc, Mutex},
        thread, time,
    };

    #[test]
    fn failure() {
        let (tx, rx) = mpsc::sync_channel(0);
        let mrx = Mutex::new(rx);

        let mut action = Action::new(move |_| {
            // Block until the message is sent, then return its value
            mrx.lock().unwrap().recv().unwrap()
        });

        for _ in 0..5 {
            assert_eq!(action.tick(&mut ()), Status::Running);
            thread::sleep(time::Duration::from_millis(100));
        }

        tx.send(Status::Failed).unwrap();

        let mut status = Status::Running;
        while status == Status::Running {
            status = action.tick(&mut ());
        }

        assert_eq!(status, Status::Failed);
    }

    #[test]
    fn success() {
        let (tx, rx) = mpsc::sync_channel(0);
        let mrx = Mutex::new(rx);

        let mut action = Action::new(move |_| {
            // Block until the message is sent, then return its value
            mrx.lock().unwrap().recv().unwrap()
        });

        for _ in 0..5 {
            assert_eq!(action.tick(&mut ()), Status::Running);
            thread::sleep(time::Duration::from_millis(100));
        }

        tx.send(Status::Succeeded).unwrap();

        let mut status = Status::Running;
        while status == Status::Running {
            status = action.tick(&mut ());
        }

        assert_eq!(status, Status::Succeeded);
    }

    #[test]
    fn inline_failure() {
        assert_eq!(
            InlineAction::new(|_| Status::Failed).tick(&mut ()),
            Status::Failed
        );
    }

    #[test]
    fn inline_success() {
        assert_eq!(
            InlineAction::new(|_| Status::Succeeded).tick(&mut ()),
            Status::Succeeded
        );
    }

    #[test]
    fn inline_running() {
        assert_eq!(
            InlineAction::new(|_| Status::Running).tick(&mut ()),
            Status::Running
        );
    }
}
