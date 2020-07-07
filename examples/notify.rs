use std::thread::sleep;
use std::time::Duration;
use ble_central::BleBuilder;
use std::io::BufRead;
use std::io::stdout;
use std::io::prelude::*;

fn main() {
    let mut ble = BleBuilder::new().unwrap().build().unwrap();
    ble.connect("C6:46:56:AC:2C:4C").unwrap();
    dbg!(ble.is_connected("C6:46:56:AC:2C:4C").unwrap());

    let mut file = ble.notify("C6_46_56_AC_2C_4C","9370002a-1bb7-1599-985b-f5e7dc991483").unwrap();

    let mut buffer = [0u8];
    loop {
        //file.read_exact(&mut buffer).unwrap();
        //print!("{}",buffer[0]);
        if file.read(&mut buffer).is_ok(){
            print!("{}",buffer[0]);
            stdout().flush().unwrap();
        }
    }

    //ble.disconnect("C6:46:56:AC:2C:4C").unwrap();
    //dbg!(ble.is_connected("C6:46:56:AC:2C:4C").unwrap());
}