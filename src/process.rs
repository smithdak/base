use std::io;
use std::time::{Duration, Instant};

use command_group::GroupChild;

/// Wait for an entire process group with a deadline.
///
/// `GroupChild::kill` terminates the POSIX process group or Windows Job Object,
/// preventing descendants that inherited captured pipes from surviving a timeout.
pub fn wait_group_timeout(
    child: &mut GroupChild,
    timeout: Duration,
    quiescent: impl Fn() -> bool,
) -> io::Result<Option<std::process::ExitStatus>> {
    let deadline = Instant::now() + timeout;
    let mut leader_status = None;
    let mut group_terminated = false;
    loop {
        if leader_status.is_none() {
            leader_status = child.try_wait()?;
        }
        if leader_status.is_some() && !group_terminated {
            // A verifier or policy command must not leave background work
            // behind. Preserve the leader's status but terminate any remaining
            // members of its process group / Windows Job Object.
            let _ = child.kill();
            group_terminated = true;
        }
        // A leader may exit while a descendant keeps inherited pipes open.
        // Treat the group as complete only after the leader has exited and all
        // caller-observed streams/writers have quiesced.
        if leader_status.is_some() && quiescent() {
            return Ok(leader_status);
        }
        let now = Instant::now();
        if now >= deadline {
            return Ok(None);
        }
        std::thread::sleep((deadline - now).min(Duration::from_millis(10)));
    }
}

pub fn wait_until(timeout: Duration, condition: impl Fn() -> bool) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        if condition() {
            return true;
        }
        let now = Instant::now();
        if now >= deadline {
            return false;
        }
        std::thread::sleep((deadline - now).min(Duration::from_millis(10)));
    }
}
