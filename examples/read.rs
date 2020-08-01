use ble_central::BleBuilder;

const DEVICE_ADDRESS: &'static str = "0A:0A:0A:0A:0A:0A";

fn main() {
    let mut ble = BleBuilder::new().build().unwrap();
    ble.connect(DEVICE_ADDRESS).unwrap();
    dbg!(ble.is_connected(DEVICE_ADDRESS).unwrap());

    let data = ble
        .read(DEVICE_ADDRESS, "93700002-1bb7-1599-985b-f5e7dc991483")
        .unwrap();
    dbg!(data);
}
