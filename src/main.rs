use std::thread;
use std::time::Duration;
use std::sync::mpsc;
use std::convert::TryInto;
use std::str::FromStr;

use btleplug::bluez::{adapter::ConnectedAdapter, manager::Manager};
use btleplug::api::{UUID, Central, Peripheral, ValueNotification};

fn get_central(manager: &Manager) -> ConnectedAdapter {
    let adapters = manager.adapters().unwrap();
    let adapter = adapters.into_iter().nth(0).unwrap();
    adapter.connect().unwrap()
}

//const CHAR_UUID: UUID = UUID::B128([132, 0, 153, 220, 231, 245, 91, 152, 153, 21, 183, 27, 175, 191, 112, 148]);

pub fn handle_notify(notification: ValueNotification, tx: mpsc::Sender<Vec<u8>>){
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
    //
    // connect to the adapter
    let central = get_central(&manager);

    // start scanning for devices
    central.start_scan().unwrap();
    // instead of waiting, you can use central.on_event to be notified of
    // new devices
    thread::sleep(Duration::from_secs(4));

    // find the device we're interested in
    let sensor = central.peripherals().into_iter()
        .find(|p| p.properties().local_name.iter()
            .inspect(|x| println!("{}",x))
            .any(|name| name.contains("ble_sensor_test"))).unwrap();

    // connect to the device
    sensor.connect().unwrap();
    dbg!(&sensor);

    // discover characteristics
    sensor.discover_characteristics().unwrap();

    let char_uuid = UUID::from_str("93:70:00:85:1B:B7:15:99:98:5B:F5:E7:DC:99:14:83").unwrap();
    // find the characteristic we want
    let chars = sensor.characteristics();
    dbg!(&char_uuid);
    dbg!(&chars);
    let test_char = chars.iter()
        .inspect(|x| println!("{}",x.uuid))
        .find(|c| c.uuid == char_uuid).unwrap();
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
            if prev_numb+1 != numb {
                break numb;
            }
        }
        prev_numb = Some(numb); 
    };
    println!("Lost a packet as the expected number was: {:?} but we recieved: {:?}",prev_numb, numb);
}