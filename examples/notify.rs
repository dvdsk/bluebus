use std::thread::sleep;
use std::time::Duration;
use ble_central::BleBuilder;

fn main() {
    let mut ble = BleBuilder::new().unwrap().build().unwrap();
    ble.connect("C6:46:56:AC:2C:4C").unwrap();
    dbg!(ble.is_connected("C6:46:56:AC:2C:4C").unwrap());

    ble.notify("C6_46_56_AC_2C_4C","9370002a-1bb7-1599-985b-f5e7dc991483").unwrap();

    sleep(Duration::from_secs(90));
    //ble.disconnect("C6:46:56:AC:2C:4C").unwrap();
    //dbg!(ble.is_connected("C6:46:56:AC:2C:4C").unwrap());
}