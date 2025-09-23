use log::*;

fn main() {
    logsy::set_echo(true);

    trace!("Library function called");
    debug!("Auth attempt");
    info!("Application has just started");
    warn!("Dereferencing null pointers harms");
    error!("This application got a boo-boo and going to be terminated");
}
