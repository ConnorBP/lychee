use std::time::Duration;
use log::info;
use memflow::prelude::*;

/// Pauses the thread with a console message
pub fn invalid_pause(name: &str) {
    info!("{} invalid. Sleeping for a bit", name);
    std::thread::sleep(Duration::from_secs(5));
}

/// Blocks thread until result returns success
pub fn wait_for<T>(result:Result<T>, delay: Duration) -> T 
{
    let ret;
    loop {
        if let Ok(val) = result {
            ret = val;
            break;
        }
        info!("waiting for valid result");
        std::thread::sleep(delay);
    }
    ret
}