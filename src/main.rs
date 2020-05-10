use std::fs::File;
use std::num::NonZeroU8;
use std::time::Duration;

use rustbus::{get_system_bus_path, MessageBuilder, Conn, RpcConn, standard_messages};
mod experiments;

//idea:
// -simple, no auth or pairing supported
// no need to explicitly connect
// builder pattern, by default pick first adapter (other is extra feature)
// use "from_raw_fd" for File (write and notify)
// store opened files in self so we close them on drop
// builder pattern for all operations

// extra ideas :
// (safety) disconnect all connected on drop [make builder option?] 

#[derive(Debug)]
enum Error {
    DbusConnectionError(rustbus::client_conn::Error),
    CouldNotConnect,
}

impl From<rustbus::client_conn::Error> for Error {
    fn from(err: rustbus::client_conn::Error) -> Error {
        Error::DbusConnectionError(err)
    }
}
impl<'a> From<rustbus::message::Message<'a,'a>> for Error {
    fn from(_: rustbus::message::Message<'a,'a>) -> Error {
        Error::CouldNotConnect
    }
}

struct BleBuilder<'a> {
    connection: RpcConn<'a,'a>,
    adapter_numb: u8,
}

impl<'a> BleBuilder<'a> {
    pub fn new() -> Result<Self, Error> {
        let session_path = get_system_bus_path()?;
        let con = Conn::connect_to_bus(session_path, true)?;
        let mut connection = RpcConn::new(con);
        // send the obligatory hello message
        connection.send_message(&mut standard_messages::hello(), None)?;

        Ok(BleBuilder {
            connection,
            adapter_numb: 0,
        })
    }

    pub fn build(self) -> Result<Ble<'a>, Error> {
        let BleBuilder{connection, adapter_numb} = self;
        
        Ok( Ble {
            connection,
            adapter_numb,
            notify_handles: Vec::new(),
        })
    }
}

struct Ble<'a> {
    //adapter
    connection: RpcConn<'a,'a>,
    adapter_numb: u8,
    notify_handles: Vec<File>,  
}

impl<'a> Ble<'a> {
    pub fn connect<T: Into<String>>(&mut self, adress: T) -> Result<(), Error> {
        let adress =adress.into().replace(":","_");

        let mut connect = MessageBuilder::new()
            .call("Connect".into())
            .at("org.bluez".into())
            .on(format!("/org/bluez/hci{}/dev_{}",self.adapter_numb, adress))
            .with_interface("org.bluez.Device1".into()) //is always Device1
            .build();

        let timeout = Some(Duration::from_secs(5));
        let response_serial = self.connection.send_message(&mut connect, timeout).unwrap();
        let msg = self.connection.wait_response(response_serial, timeout).unwrap();
        
        match msg.typ {
            rustbus::message::MessageType::Reply => Ok(()),
            rustbus::message::MessageType::Error => Err(Error::from(msg)),
            _ => { 
                let dbg_str = format!("Connect can only be awnserd 
                    with Error or Reply however we got: {:?}", &msg);
                dbg!(&dbg_str);
                panic!();
            }
        }
    }
    
    pub fn disconnect<T: Into<String>>(&mut self, adress: T) -> Result<(), Error> {
        let adress =adress.into().replace(":","_");

        let mut connect = MessageBuilder::new()
            .call("Disconnect".into())
            .at("org.bluez".into())
            .on(format!("/org/bluez/hci{}/dev_{}",self.adapter_numb, adress))
            .with_interface("org.bluez.Device1".into()) //is always Device1
            .build();

        let timeout = Some(Duration::from_secs(5));
        let response_serial = self.connection.send_message(&mut connect, timeout).unwrap();
        let msg = self.connection.wait_response(response_serial, timeout).unwrap();
        
        match msg.typ {
            rustbus::message::MessageType::Reply => Ok(()),
            rustbus::message::MessageType::Error => Err(Error::from(msg)),
            _ => { 
                let dbg_str = format!("Connect can only be awnserd 
                    with Error or Reply however we got: {:?}", &msg);
                dbg!(&dbg_str);
                panic!();
            }
        }
    }
}


use std::thread::sleep;
fn main() {
    let mut ble = BleBuilder::new().unwrap().build().unwrap();
    ble.connect("C6:46:56:AC:2C:4C").unwrap();

    sleep(Duration::from_secs(30));
    ble.disconnect("C6:46:56:AC:2C:4C").unwrap();
    //experiments::exp().unwrap();
}