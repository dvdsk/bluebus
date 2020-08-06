use bluebus::BleBuilder;
use nix::poll::{poll, ppoll, PollFd, PollFlags};
use nix::sys::signal::SigSet;
use std::fs::File;
use std::io::Read;
use std::os::unix::io::FromRawFd;

//use epoll

const DEVICE_ADDRESS: &'static str = "0A:0A:0A:0A:0A:0A";

fn main() {
    let mut ble = BleBuilder::new().build().unwrap();
    ble.connect(DEVICE_ADDRESS).unwrap();
    dbg!(ble.is_connected(DEVICE_ADDRESS).unwrap());

    let mut fd = ble
        .notify(DEVICE_ADDRESS, "93700001-1bb7-1599-985b-f5e7dc991483")
        .unwrap();
    //ble.listen_dbus();

    let mut buffer = [0u8; 4];
    let pollfd = PollFd::new(fd, PollFlags::all());
    let mut file = unsafe { File::from_raw_fd(fd) };
    loop {
        if ppoll(&mut [pollfd], None, SigSet::all()).unwrap() != 0 {
            //if poll(&mut [pollfd], -1).unwrap() != 0 {

            if let Some(event) = pollfd.revents() {
                if event.is_empty() {
                    continue;
                }

                dbg!(event);
                let nread = file.read(&mut buffer).unwrap();
                println!("nread: {}", nread);
            }
        }
    }
}
