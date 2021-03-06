extern crate tokio_io;
extern crate tokio_process;
extern crate futures;
extern crate tokio_core;

use tokio_core::reactor::Core;
use tokio_core::reactor::Handle;
use tokio_process::CommandExt;
use tokio_io::io::lines;
use futures::Stream;
use std::io;
use std::process::{Command, Stdio};

// Create a new command from the given command and arguments.
fn cmd<'a, I>(c: &str, args: I) -> Command
where
    I: IntoIterator<Item = &'a str>,
{
    let mut cmd = ::std::process::Command::new(c);
    cmd.args(args);
    cmd
}

// Spawn a child that runs the given command and return a line-by-line
// stream of its stdout. Stderr is ignored.
fn cmd_stdout(mut cmd: Command, handle: &Handle) -> Box<Stream<Item = String, Error = io::Error>> {
    // Let us read stdout and ignore stderr
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::null());

    // Spawn the child
    let mut child = cmd.spawn_async(handle).expect("spawning child to succeed");
    let id = child.id();

    // Create a line-based stream from stdout of child
    let stdout = child.stdout().take().expect("to get stdout handle");
    let reader = ::std::io::BufReader::new(stdout);
    let stream = lines(reader).map(move |line| format!("[CHILD {}] {}", id, line));

    // Make sure the child process survives the call to the 'child' variable destructor
    // If we did not call this, child process would be killed if cmd_stdout(..) returns.
    child.forget();

    Box::new(stream)
}


fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let s1 = cmd_stdout(cmd("ping", vec!["127.0.0.1"]), &handle);
    let s2 = cmd_stdout(cmd("ping", vec!["0.0.0.0"]), &handle);

    let h = s1.select(s2).for_each(|line| {
        println!("LINE: {}", line);
        ::futures::future::ok(())
    });

    core.run(h).unwrap();
}
