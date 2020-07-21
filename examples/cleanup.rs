use ble_central::BleBuilder;

fn main() {
    let mut ble = BleBuilder::new().unwrap().build().unwrap();
    ble.remove("0A:0A:0A:0A:0A:0A").unwrap();
}