use ble_central::BleBuilder;
use std::io::prelude::*;
use std::time::Instant;
use nix::poll::{PollFd, PollFlags, poll};
use std::os::unix::io::AsRawFd;

fn main() {
    let mut ble = BleBuilder::new().unwrap().build().unwrap();
    ble.connect("C6:46:56:AC:2C:4C").unwrap();
    dbg!(ble.is_connected("C6:46:56:AC:2C:4C").unwrap());

    let mut file = ble.notify("C6_46_56_AC_2C_4C","9370002a-1bb7-1599-985b-f5e7dc991483").unwrap();

    let mut counter = 0u32;
    let mut start = Instant::now();

    let mut buffer = [0u8; 4];
    let mut expected = None;
    let pollfd = PollFd::new(file.as_raw_fd(), PollFlags::POLLIN);
    loop {
        if let Err(_) = poll(&mut [pollfd], -1){
            continue;
        }
        let nread = file.read(&mut buffer).unwrap();
        //if file.read(&mut buffer).is_err(){
        //    continue;
        //}
        if nread != 4 {
            println!("nread: {}", nread);
        }

        let new = u32::from_le_bytes(buffer);
        if let Some(expected) = expected {
            if new != expected {
                println!("error: new != prev+1, {} != {}",new, expected);
            }
        }
        expected = Some(new+1);
        //expected = expected.map_or(Some(new+1), |e| Some(e+1));


        counter += 1;
        if counter == 10_000 {
            let freq = (counter as f32)/start.elapsed().as_secs_f32();
            println!("recieved {} numbers at {} hz", counter, freq);
            counter = 0;
            start = Instant::now();
        }

    }

    //ble.disconnect("C6:46:56:AC:2C:4C").unwrap();
    //dbg!(ble.is_connected("C6:46:56:AC:2C:4C").unwrap());
}