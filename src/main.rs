mod dbus;
mod library;
mod source;
use anyhow::Result;
use nix::sys::signal::{sigaction, SaFlags, SigAction, SigHandler, SigSet, Signal};

fn main() -> Result<()> {
    let act = SigAction::new(SigHandler::SigIgn, SaFlags::empty(), SigSet::all());
    unsafe {
        sigaction(Signal::SIGCHLD, &act).expect("signal setup failed!");
    }
    dbus::serve()
}
