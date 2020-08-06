use bluebus::BleBuilder;
use std::thread;
use std::time::Duration;

const DEVICE_ADDRESS: &'static str = "0A:0A:0A:0A:0A:A0";

fn main() {
    let mut ble = BleBuilder::new().build().unwrap();
    ble.remove(DEVICE_ADDRESS).unwrap();
    ble.remove_attribute_cache(DEVICE_ADDRESS).unwrap();

    ble.start_discovery().unwrap();
    while ble.connect(DEVICE_ADDRESS).is_err() {
        thread::sleep(Duration::from_secs(1));
    }
    ble.stop_discovery().unwrap();
}
