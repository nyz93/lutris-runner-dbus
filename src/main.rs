mod dbus;
mod lutris;
use anyhow::Result;

fn main() -> Result<()> {
    dbus::serve()
}
