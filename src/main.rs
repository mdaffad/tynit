use nix::sys::signal::{self, SaFlags, SigAction, SigHandler, SigSet, Signal};
use nix::unistd::Pid;
use std::env;
use std::process::{exit, Command};

static mut CHILD_PID: Option<i32> = None;

extern "C" fn handle_signal(signum: i32) {
    unsafe {
        if let Some(pid) = CHILD_PID {
            if let Ok(sig) = Signal::try_from(signum) {
                let _ = signal::kill(Pid::from_raw(pid), sig);
            } else {
                eprintln!("Invalid signal number: {}", signum);
            }
        }
        exit(1);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <command> [args...]", args[0]);
        exit(1);
    }

    let command = &args[1];
    let command_args = &args[2..];

    unsafe {
        let sa = SigAction::new(
            SigHandler::Handler(handle_signal),
            SaFlags::SA_RESTART,
            SigSet::empty(),
        );

        let _ = signal::sigaction(Signal::SIGINT, &sa);
        let _ = signal::sigaction(Signal::SIGTERM, &sa);
    }

    match Command::new(command).args(command_args).spawn() {
        Ok(mut child) => {
            let status = child.wait().expect("Failed to wait on child");
            println!("Child exited with status: {}", status);
        }
        Err(e) => {
            eprintln!("Failed to start process: {}", e);
            exit(1);
        }
    }
}
