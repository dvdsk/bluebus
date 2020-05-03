use std::convert::TryInto;
use std::str::FromStr;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use btleplug::api::{BDAddr, Central, Peripheral, ValueNotification, UUID};
use btleplug::bluez::{adapter::ConnectedAdapter, manager::Manager};
use simplelog::{SimpleLogger, LevelFilter, Config, ConfigBuilder};

fn get_central(manager: &Manager) -> ConnectedAdapter {
    let adapters = manager.adapters().unwrap();
    let adapter = adapters.into_iter().nth(0).unwrap();
    adapter.connect().unwrap()
}

const BASE_UUID: [u8;16] = [
    131, 20, 153, 220, 231, 245, 91, 152, 153, 21, 183, 27, 175, 191, 112, 147
];
const TARGET_DEV: BDAddr = BDAddr {
    address: [0x4C, 0x2C, 0xAC, 0x56, 0x46, 0xC6],
}; //reverse of what is normally displayed

//matches the nrf api approch
fn char_uuid_from(base: [u8; 16], service: u16) -> UUID {
    let mut final_uuid = [0u8; 16];
    final_uuid = base;
    //let service = service.to_be_bytes();
    let service = service.to_le_bytes();
    final_uuid[12] = service[0];
    final_uuid[13] = service[1];
    UUID::B128(final_uuid)
}

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

    let config = ConfigBuilder::new()
        .add_filter_ignore_str("btleplug::bluez::adapter")
        .build();
    let _ = SimpleLogger::init(LevelFilter::Trace, config);

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
        .find(|p| p.properties().address == TARGET_DEV)
        .unwrap();

    // connect to the device
    sensor.connect().unwrap();

    // discover characteristics
    sensor.discover_characteristics().unwrap();

    // find the characteristic we want
    let char_uuid = char_uuid_from(BASE_UUID, 42);
    let chars = dbg!(sensor.characteristics());
    dbg!(char_uuid);
    let test_char = chars
        .iter()
        .inspect(|x| println!("{}", x.uuid))
        .find(|c| c.uuid == char_uuid)
        .unwrap();
    dbg!(&test_char);
    sensor.on_notification(Box::new(move |value| handle_notify(value, tx.clone())));
    dbg!("trying to subscribe");
    sensor.subscribe(&test_char).unwrap();
    dbg!("subscribed succesfully");
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
