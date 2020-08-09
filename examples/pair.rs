use std::io;
use std::time::Duration;

use bluebus::BleBuilder;

const DEVICE_ADDRESS: &'static str = "0A:0A:0A:0A:0A:0A";

fn main() {
    let mut ble = BleBuilder::default().build().unwrap();
    ble.connect(DEVICE_ADDRESS).unwrap();
    dbg!(ble.is_connected(DEVICE_ADDRESS).unwrap());

    //let get_key = || 123456;
    let get_key = || {
        let mut input = String::new();
        println!("enter 6 digit passkey");
        io::stdin().read_line(&mut input).unwrap();
        input.trim().parse().unwrap()
    };

    if !ble.is_paired(DEVICE_ADDRESS).unwrap() {
        ble.pair(DEVICE_ADDRESS, get_key, Duration::from_secs(5))
            .unwrap();
    } else {
        println!("already paired");
    }
}
