mod characteristic;
mod device;

use crate::Ble;
use rustbus::client_conn::Timeout;

impl Ble {
    pub fn listen_dbus(&mut self){
        loop {
            let messg = self
                .connection
                .wait_signal(Timeout::Infinite)//wait_call(Timeout::Infinite)
                .unwrap();
            
            dbg!(messg.unmarshall_all().unwrap());
        }
    }
}