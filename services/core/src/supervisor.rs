use std::io;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use limen_bridge_sdk::LaunchPlan;

const POLL_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcessExit {
    Normal,
    Crashed { exit_code: Option<i32> },
    TimedOut,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ProcessSupervisor;

impl ProcessSupervisor {
    pub fn start(self, plan: &LaunchPlan) -> io::Result<ManagedProcess> {
        let mut command = Command::new(plan.executable());
        command
            .args(plan.argv())
            .env_clear()
            .envs(plan.environment())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        command.spawn().map(ManagedProcess::new)
    }
}

#[derive(Debug)]
pub struct ManagedProcess {
    child: Child,
    reaped: bool,
}

impl ManagedProcess {
    fn new(child: Child) -> Self {
        Self {
            child,
            reaped: false,
        }
    }

    pub fn wait(
        &mut self,
        timeout: Duration,
        cancelled: &AtomicBool,
    ) -> io::Result<ProcessExit> {
        let started_at = Instant::now();

        loop {
            if let Some(status) = self.child.try_wait()? {
                self.reaped = true;
                return Ok(if status.success() {
                    ProcessExit::Normal
                } else {
                    ProcessExit::Crashed {
                        exit_code: status.code(),
                    }
                });
            }

            if cancelled.load(Ordering::Acquire) {
                self.terminate_and_reap()?;
                return Ok(ProcessExit::Cancelled);
            }

            if started_at.elapsed() >= timeout {
                self.terminate_and_reap()?;
                return Ok(ProcessExit::TimedOut);
            }

            std::thread::sleep(POLL_INTERVAL);
        }
    }

    fn terminate_and_reap(&mut self) -> io::Result<()> {
        if self.reaped {
            return Ok(());
        }

        match self.child.kill() {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::InvalidInput => {}
            Err(error) => return Err(error),
        }
        let _status = self.child.wait()?;
        self.reaped = true;
        Ok(())
    }
}

impl Drop for ManagedProcess {
    fn drop(&mut self) {
        if !self.reaped {
            let _ = self.child.kill();
            let _ = self.child.wait();
            self.reaped = true;
        }
    }
}
