use std::os::unix::io::FromRawFd;
use std::fs::File;
use std::num::NonZeroU8;
use std::time::Duration;
use std::os::unix::io::RawFd;

use rustbus::params::Container;
use rustbus::{get_system_bus_path, MessageBuilder, Conn, RpcConn, standard_messages};
mod experiments;
mod error;
use error::{Error, to_error, ErrorContext};
mod dbus_helpers;
use dbus_helpers::{unwrap_container, unwrap_variant, unwrap_dict, unwrap_string, unwrap_objectpath, unwrap_base};
//idea:
// -simple, no auth or pairing supported
// no need to explicitly connect
// builder pattern, by default pick first adapter (other is extra feature)
// use "from_raw_fd" for File (write and notify)
// store opened files in self so we close them on drop
// builder pattern for all operations

// extra ideas :
// (safety) disconnect all connected on drop [make builder option?] 

const TIMEOUT: Option<Duration> = Some(Duration::from_secs(5));

pub struct BleBuilder<'a> {
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
        })
    }
}

pub struct Ble<'a> {
    //adapter
    connection: RpcConn<'a,'a>,
    adapter_numb: u8,  
}

impl<'a> Ble<'a> {

    #[allow(dead_code)]
    pub fn connect(&mut self, adress: impl Into<String>) -> Result<(), Error> {
        let adress =adress.into().replace(":","_");

        let mut connect = MessageBuilder::new()
            .call("Connect".into())
            .at("org.bluez".into())
            .on(format!("/org/bluez/hci{}/dev_{}",self.adapter_numb, adress))
            .with_interface("org.bluez.Device1".into()) //is always Device1
            .build();

        let response_serial = self.connection.send_message(&mut connect, TIMEOUT).unwrap();
        let msg = self.connection.wait_response(response_serial, TIMEOUT).unwrap();
        
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

    #[allow(dead_code)]
    pub fn disconnect(&mut self, adress: impl Into<String>) -> Result<(), Error> {
        let adress =adress.into().replace(":","_");

        let mut connect = MessageBuilder::new()
            .call("Disconnect".into())
            .at("org.bluez".into())
            .on(format!("/org/bluez/hci{}/dev_{}",self.adapter_numb, adress))
            .with_interface("org.bluez.Device1".into()) //is always Device1
            .build();

        let response_serial = self.connection.send_message(&mut connect, TIMEOUT).unwrap();
        let msg = self.connection.wait_response(response_serial, TIMEOUT).unwrap();
        
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

    #[allow(dead_code)]
    pub fn is_connected(&mut self, adress: impl AsRef<str>)
     -> Result<bool, Error>{

        let mut isConnected = MessageBuilder::new()
            .call("Get".into())
            .at("org.bluez".into())
            .on(format!("/org/bluez/hci{}/dev_{}",self.adapter_numb, adress.as_ref()))
            .with_interface("org.freedesktop.DBus.Properties".into()); //is always Device1
            isConnected.add_param2("org.bluez.Device1","Connected");
        let mut isConnected = isConnected.build();
        
        let response_serial = self.connection.send_message(&mut isConnected, TIMEOUT).unwrap();
        let reply = self.connection.wait_response(response_serial, TIMEOUT).unwrap();
        dbg!(reply);
        Ok(true)
    }

    #[allow(dead_code)]
    pub fn notify(&mut self, adress: impl Into<String>, uuid: impl AsRef<str>)
     -> Result<File,Error> {

        let char_path = self.path_for_char(adress, uuid)?
            .ok_or(Error::CharacteristicNotFound)?;

        let mut aquire_notify = MessageBuilder::new()
            .call("AcquireNotify".into())
            .at("org.bluez".into())
            .on(char_path.clone())
            .with_interface("org.bluez.GattCharacteristic1".into()) //is always GattCharacteristic1
            .build();

        dbg!(&aquire_notify);

        let response_serial = self.connection.send_message(&mut aquire_notify, TIMEOUT).unwrap();
        let reply = self.connection.wait_response(response_serial, TIMEOUT).unwrap();
        match &reply.typ {
            rustbus::message::MessageType::Error => 
                return Err(to_error(reply, ErrorContext::AquireNotify(char_path))),
            rustbus::message::MessageType::Reply => (),
            _ => Err(Error::UnexpectedDbusReply)?,
        }

        let rustbus::message::Message{mut raw_fds, ..} = reply;
        let raw_fd = raw_fds.pop().ok_or(Error::NoFdReturned)?;
        let file = unsafe {File::from_raw_fd(raw_fd)};
        Ok(file)
    }

    fn path_for_char(&mut self, adress: impl Into<String>, char_uuid: impl AsRef<str>)
     -> Result<Option<String>, Error> {

        let mut get_paths = MessageBuilder::new()
            .call("GetManagedObjects".into())
            .at("org.bluez".into())
            .on("/".into())
            .with_interface("org.freedesktop.DBus.ObjectManager".into())
            .build();
        
        let response_serial = self.connection.send_message(&mut get_paths, TIMEOUT).unwrap();
        let mut reply = self.connection.wait_response(response_serial, TIMEOUT).unwrap();

        let param = reply.params.pop().unwrap();
        let container = unwrap_container(param).unwrap();
        let dict = unwrap_dict(container).unwrap();
        
        let device_path = format!("/org/bluez/hci{}/dev_{}", 
            self.adapter_numb,
            adress.into().replace(":","_"));
        
        for (path, base) in dict
            .into_iter()
            .filter_map(unwrap_objectpath)
            .filter(|(p,_)| p.contains(&device_path))
            .filter(|(p,_)| p.contains("char"))
            .filter(|(p,_)| !p.contains("desc")){

            let container = unwrap_container(base).unwrap();
            let mut dict = unwrap_dict(container).unwrap();
            let gatt_char = dict.remove(&rustbus::Base::String("org.bluez.GattCharacteristic1".into()))
                .expect("char object path should always have GattCharacteristic1");
            let gatt_char = unwrap_container(gatt_char).unwrap();
            let mut gatt_char = unwrap_dict(gatt_char).unwrap();
            let uuid = gatt_char.remove(&rustbus::Base::String("UUID".into()))
                .expect("char object should always have a UUID");
            let uuid = unwrap_container(uuid).unwrap();
            let uuid = unwrap_variant(uuid).unwrap();
            let uuid = uuid.value;
            let uuid = unwrap_base(uuid).unwrap();
            let uuid = unwrap_string(uuid).unwrap();
            
            if &uuid == char_uuid.as_ref() {
                return Ok(Some(path));
            }
        }
        Ok(None)
    }

}