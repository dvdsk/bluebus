use std::convert::TryInto;
use std::str::FromStr;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use rumble::api::{BDAddr, Central, Peripheral, ValueNotification, UUID};
use rumble::bluez::{adapter::ConnectedAdapter, manager::Manager};

fn get_central(manager: &Manager) -> ConnectedAdapter {
    let adapters = manager.adapters().unwrap();
    let adapter = adapters.into_iter().nth(0).unwrap();
    adapter.connect().unwrap()
}

const char_uuid: UUID = UUID::B128([
    132, 0, 153, 220, 231, 245, 91, 152, 153, 21, 183, 27, 175, 191, 112, 148,
]);
const target_dev: BDAddr = BDAddr {
    address: [0x4C, 0x2C, 0xAC, 0x56, 0x46, 0xC6],
}; //reverse of what is normally displayed

pub fn handle_notify(notification: ValueNotification, tx: mpsc::Sender<Vec<u8>>) {
    dbg!("handling notify");
    tx.send(notification.value).unwrap();
}

fn read_be_u32(input: &mut &[u8]) -> u32 {
    let (int_bytes, rest) = input.split_at(std::mem::size_of::<u32>());
    *input = rest;
    u32::from_be_bytes(int_bytes.try_into().unwrap())
}

pub fn main() {
    let (tx, rx) = mpsc::channel();
    let manager = Manager::new().unwrap();

    // get the first bluetooth adapter
    let adapters = manager.adapters().unwrap();
    let mut adapter = adapters.into_iter().nth(0).unwrap();

    // reset the adapter -- clears out any errant state
    adapter = manager.down(&adapter).unwrap();
    adapter = manager.up(&adapter).unwrap();

    // connect to the adapter
    let central = adapter.connect().unwrap();
    dbg!("connected to adapter");

    // start scanning for devices
    central.start_scan().unwrap();
    // instead of waiting, you can use central.on_event to be notified of
    // new devices
    thread::sleep(Duration::from_secs(4));

    // find the device we're interested in
    let sensor = central
        .peripherals()
        .into_iter()
        //.inspect(|p| println!("{:?}", p.properties().address))
        .find(|p| p.properties().address == target_dev)
        .unwrap();

    // connect to the device
    dbg!(&sensor);
    sensor.connect().unwrap();
    dbg!(&sensor);

    // discover characteristics
    sensor.discover_characteristics().unwrap();

    //let char_uuid = UUID::from_str("93:70:00:85:1B:B7:15:99:98:5B:F5:E7:DC:99:14:83").unwrap();
    // find the characteristic we want
    let chars = sensor.characteristics();
    dbg!(&char_uuid);
    dbg!(&chars);
    let test_char = chars
        .iter()
        .inspect(|x| println!("{}", x.uuid))
        .find(|c| c.uuid == char_uuid)
        .unwrap();
    dbg!();
    sensor.on_notification(Box::new(move |value| handle_notify(value, tx.clone())));
    dbg!();
    sensor.subscribe(&test_char).unwrap();
    dbg!();
    let mut prev_numb = None;
    let numb = loop {
        dbg!("loop loop");
        let recieved: Vec<u8> = rx.recv().unwrap();
        let numb = read_be_u32(&mut &recieved[0..4]);

        if let Some(prev_numb) = prev_numb {
            if prev_numb + 1 != numb {
                break numb;
            }
        }
        prev_numb = Some(numb);
    };
    println!(
        "Lost a packet as the expected number was: {:?} but we recieved: {:?}",
        prev_numb, numb
    );
}
