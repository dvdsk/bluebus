use std::os::unix::io::FromRawFd;
use std::fs::File;
use std::num::NonZeroU8;
use std::time::Duration;
use std::os::unix::io::RawFd;

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

const TIMEOUT: Option<Duration> = Some(Duration::from_secs(5));

#[derive(Debug)]
enum Error {
    DbusConnectionError(rustbus::client_conn::Error),
    CouldNotConnect,
    UuidNotFound,
    DeviceNotFound,
    NoFdReturned,
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
        })
    }
}

struct Ble<'a> {
    //adapter
    connection: RpcConn<'a,'a>,
    adapter_numb: u8,  
}

impl<'a> Ble<'a> {
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

    pub fn notify(&mut self, adress: impl Into<String>, uuid: impl AsRef<str>)
     -> Result<File,Error> {

        let char_path = self.path_for_char(adress, uuid)?;

        let mut aquire_notify = MessageBuilder::new()
            .call("AcquireNotify".into())
            .at("org.bluez".into())
            .on(char_path)
            .with_interface("org.bluez.GattCharacteristic1".into()) //is always GattCharacteristic1
            .build();

        let response_serial = self.connection.send_message(&mut aquire_notify, TIMEOUT).unwrap();
        let msg = self.connection.wait_response(response_serial, TIMEOUT).unwrap();
        dbg!(&msg);

        let rustbus::message::Message{mut raw_fds, ..} = msg;
        let raw_fd = raw_fds.pop().ok_or(Error::NoFdReturned)?;
        let file = unsafe {File::from_raw_fd(raw_fd)};
        Ok(file)
    }

    fn path_for_char(&mut self, adress: impl Into<String>, uuid: impl AsRef<str>)
     -> Result<String, Error> {

        let mut get_paths = MessageBuilder::new()
            .call("GetManagedObjects".into())
            .at("org.bluez".into())
            .on("/".into())
            .with_interface("org.freedesktop.DBus.ObjectManager".into())
            .build();
        
        let response_serial = self.connection.send_message(&mut get_paths, TIMEOUT).unwrap();
        let msg = self.connection.wait_response(response_serial, TIMEOUT).unwrap();
        dbg!(msg);
        //reply unpack
        let output = "";

        let device_path = format!("\n'/org/bluez/hci{}/dev_{}", 0, adress.into().replace(":","_"));
        let uuid_start = output.find(uuid.as_ref()).ok_or(Error::UuidNotFound).unwrap();
        let path_start = output[..uuid_start]
            .rfind(&device_path)
            .ok_or(Error::DeviceNotFound).unwrap() + "\n'".len();
        
        let path = output[path_start..].splitn(2,"':").nth(0).unwrap();
        Ok(path.to_owned())
    }

}


use std::thread::sleep;
fn main() {
    let mut ble = BleBuilder::new().unwrap().build().unwrap();
    ble.connect("C6:46:56:AC:2C:4C").unwrap();

    ble.notify("C6_46_56_AC_2C_4C","9370002a-1bb7-1599-985b-f5e7dc991483").unwrap();

    sleep(Duration::from_secs(30));
    ble.disconnect("C6:46:56:AC:2C:4C").unwrap();
    //experiments::exp().unwrap();
}



#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn extract_path_from_output() {
        let output = teststr;
        let uuid = "9370002a-1bb7-1599-985b-f5e7dc991483";
        let device_path = format!("\n'/org/bluez/hci{}/dev_{}", 0, "C6_46_56_AC_2C_4C");
        
        let uuid_start = output.find(uuid).ok_or(Error::UuidNotFound).unwrap();
        let path_start = output[..uuid_start]
            .rfind(&device_path)
            .ok_or(Error::DeviceNotFound).unwrap() + "\n'".len();
        
        let path = output[path_start..].splitn(2,"':").nth(0).unwrap();
        
        assert_eq!(path,
            "/org/bluez/hci0/dev_C6_46_56_AC_2C_4C/service000e/char000f"
        );
    }



    const teststr: &str = r#"{'/org/bluez': {'org.bluez.AgentManager1': {},
    'org.bluez.HealthManager1': {},
    'org.bluez.ProfileManager1': {},
    'org.freedesktop.DBus.Introspectable': {}},
'/org/bluez/hci0': {'org.bluez.Adapter1': {'Address': 'B8:27:EB:3C:7A:4E',
                                'AddressType': 'public',
                                'Alias': 'raspberrypi',
                                'Class': 0,
                                'Discoverable': False,
                                'DiscoverableTimeout': 180,
                                'Discovering': False,
                                'Modalias': 'usb:v1D6Bp0246d0532',
                                'Name': 'raspberrypi',
                                'Pairable': True,
                                'PairableTimeout': 0,
                                'Powered': True,
                                'UUIDs': ['00001801-0000-1000-8000-00805f9b34fb',
                                          '0000110e-0000-1000-8000-00805f9b34fb',
                                          '00001200-0000-1000-8000-00805f9b34fb',
                                          '0000110c-0000-1000-8000-00805f9b34fb',
                                          '00001800-0000-1000-8000-00805f9b34fb']},
         'org.bluez.GattManager1': {},
         'org.bluez.LEAdvertisingManager1': {'ActiveInstances': 0,
                                             'SupportedIncludes': ['tx-power',
                                                                   'appearance',
                                                                   'local-name'],
                                             'SupportedInstances': 5},
         'org.bluez.Media1': {},
         'org.bluez.NetworkServer1': {},
         'org.freedesktop.DBus.Introspectable': {},
         'org.freedesktop.DBus.Properties': {}},
'/org/bluez/hci0/dev_C6_46_56_AC_2C_4C': {'org.bluez.Device1': {'Adapter': '/org/bluez/hci0',
                                                     'Address': 'C6:46:56:AC:2C:4C',
                                                     'AddressType': 'random',
                                                     'Alias': 'ble_sensor_test',
                                                     'Blocked': False,
                                                     'Connected': False,
                                                     'LegacyPairing': False,
                                                     'Name': 'ble_sensor_test',
                                                     'Paired': False,
                                                     'ServicesResolved': False,
                                                     'Trusted': False,
                                                     'UUIDs': ['00001800-0000-1000-8000-00805f9b34fb',
                                                               '00001801-0000-1000-8000-00805f9b34fb',
                                                               '93700000-1bb7-1599-985b-f5e7dc991483']},
                               'org.freedesktop.DBus.Introspectable': {},
                               'org.freedesktop.DBus.Properties': {}},
'/org/bluez/hci0/dev_C6_46_56_AC_2C_4C/service000a': {'org.bluez.GattService1': {'Device': '/org/bluez/hci0/dev_C6_46_56_AC_2C_4C',
                                                                      'Includes': [],
                                                                      'Primary': True,
                                                                      'UUID': '00001801-0000-1000-8000-00805f9b34fb'},
                                           'org.freedesktop.DBus.Introspectable': {},
                                           'org.freedesktop.DBus.Properties': {}},
'/org/bluez/hci0/dev_C6_46_56_AC_2C_4C/service000a/char000b': {'org.bluez.GattCharacteristic1': {'Flags': ['indicate'],
                                                                                      'Notifying': False,
                                                                                      'Service': '/org/bluez/hci0/dev_C6_46_56_AC_2C_4C/service000a',
                                                                                      'UUID': '00002a05-0000-1000-8000-00805f9b34fb',
                                                                                      'Value': []},
                                                    'org.freedesktop.DBus.Introspectable': {},
                                                    'org.freedesktop.DBus.Properties': {}},
'/org/bluez/hci0/dev_C6_46_56_AC_2C_4C/service000a/char000b/desc000d': {'org.bluez.GattDescriptor1': {'Characteristic': '/org/bluez/hci0/dev_C6_46_56_AC_2C_4C/service000a/char000b',
                                                                                           'UUID': '00002902-0000-1000-8000-00805f9b34fb',
                                                                                           'Value': []},
                                                             'org.freedesktop.DBus.Introspectable': {},
                                                             'org.freedesktop.DBus.Properties': {}},
'/org/bluez/hci0/dev_C6_46_56_AC_2C_4C/service000e': {'org.bluez.GattService1': {'Device': '/org/bluez/hci0/dev_C6_46_56_AC_2C_4C',
                                                                      'Includes': [],
                                                                      'Primary': True,
                                                                      'UUID': '93700000-1bb7-1599-985b-f5e7dc991483'},
                                           'org.freedesktop.DBus.Introspectable': {},
                                           'org.freedesktop.DBus.Properties': {}},
'/org/bluez/hci0/dev_C6_46_56_AC_2C_4C/service000e/char000f': {'org.bluez.GattCharacteristic1': {'Flags': ['read',
                                                                                                'notify'],
                                                                                      'NotifyAcquired': False,
                                                                                      'Notifying': False,
                                                                                      'Service': '/org/bluez/hci0/dev_C6_46_56_AC_2C_4C/service000e',
                                                                                      'UUID': '9370002a-1bb7-1599-985b-f5e7dc991483',
                                                                                      'Value': []},
                                                    'org.freedesktop.DBus.Introspectable': {},
                                                    'org.freedesktop.DBus.Properties': {}},
'/org/bluez/hci0/dev_C6_46_56_AC_2C_4C/service000e/char000f/desc0011': {'org.bluez.GattDescriptor1': {'Characteristic': '/org/bluez/hci0/dev_C6_46_56_AC_2C_4C/service000e/char000f',
                                                                                           'UUID': '00002902-0000-1000-8000-00805f9b34fb',
                                                                                           'Value': []},
                                                             'org.freedesktop.DBus.Introspectable': {},
                                                             'org.freedesktop.DBus.Properties': {}}}"#;



}