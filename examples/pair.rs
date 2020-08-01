use ble_central::BleBuilder;

const DEVICE_ADDRESS: &'static str = "0A:0A:0A:0A:0A:0A";

fn main() {
    let mut ble = BleBuilder::new().unwrap().build().unwrap();
    ble.connect(DEVICE_ADDRESS).unwrap();
    dbg!(ble.is_connected(DEVICE_ADDRESS).unwrap());

    ble.pair(DEVICE_ADDRESS).unwrap();
}