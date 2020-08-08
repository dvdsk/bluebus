use bluebus::BleBuilder;
use bluebus::{Context, Error};
use std::io::ErrorKind;
use std::thread;
use std::time::Duration;

const DEVICE_ADDRESS: &'static str = "0A:0A:0A:0A:0A:A0";

//TODO debug!

fn main() {
    let mut ble = BleBuilder::new().build().unwrap();
    if let Err(e) = ble.remove(DEVICE_ADDRESS) {
        if e == Error::DoesNotExist(Context::Remove) {
            println!("could not find device, already removed?");
        } else {
            panic!("error: {:?}", e);
        }
    }

    if let Err(e) = ble.remove_attribute_cache(DEVICE_ADDRESS) {
        match e {
            Error::CouldNotRemoveCache(io_error) => {
                if io_error.kind() != ErrorKind::NotFound {
                    panic!("io error: {:?}", &io_error);
                }
            }
            _ => panic!("error: {:?}", e),
        }
    }

    ble.start_discovery().unwrap();
    while ble.connect(DEVICE_ADDRESS).is_err() {
        println!("could not connect to {}, retrying...", DEVICE_ADDRESS);
        thread::sleep(Duration::from_secs(5));
    }
    ble.stop_discovery().unwrap();
}
