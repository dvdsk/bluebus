use ble_central::BleBuilder;

const DEVICE_ADDRESS: &'static str = "0A:0A:0A:0A:0A:0A";

fn main() {
    let mut ble = BleBuilder::new().unwrap().build().unwrap();
    ble.connect(DEVICE_ADDRESS).unwrap();

    ble.write(DEVICE_ADDRESS, 
        "93700003-1bb7-1599-985b-f5e7dc991483", 
        vec![1,2,3,4]).unwrap();
}