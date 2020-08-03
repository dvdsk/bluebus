//use std::fs::File;
use std::collections::HashMap;
use std::fs::File;
use std::os::unix::io::FromRawFd;
use std::time::{Duration, Instant};

pub use rustbus::client_conn::Timeout;
use rustbus::message_builder::MarshalledMessage;
use rustbus::params::message;
use rustbus::{get_system_bus_path, params, standard_messages, Conn, MessageBuilder, RpcConn};

mod error;
use error::to_error;
pub use error::Error;

pub mod dbus_helpers;
use dbus_helpers::*;
use rustbus::wire::marshal::traits::ObjectPath;

pub mod util;

pub struct BleBuilder {
    adapter_numb: u8,
    timeout: Timeout,
}

impl BleBuilder {
    pub fn new() -> Self {
        BleBuilder {
            adapter_numb: 0,
            timeout: Timeout::Duration(Duration::from_secs(5)),
        }
    }

    pub fn with_timeout(mut self, timeout: Timeout) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn build(self) -> Result<Ble, Error> {
        let session_path = get_system_bus_path()?;
        let con = Conn::connect_to_bus(session_path, true)?;
        let mut connection = RpcConn::new(con);
        // send the obligatory hello message
        let response_serial =
            connection.send_message(&mut standard_messages::hello(), Timeout::Infinite)?;
        let mut reply = connection
            .wait_response(response_serial, self.timeout)?
            .unmarshall_all()?;
        let param = reply.params.pop().unwrap();
        let container = unwrap_base(param).unwrap();
        let conn_name = unwrap_string(container).unwrap();

        let mut message = get_name_owner("org.bluez".to_owned())?;
        let response_serial = connection.send_message(&mut message, self.timeout)?;
        let msg = connection
            .wait_response(response_serial, self.timeout)?
            .unmarshall_all()?;
        dbg!(msg);

        let mut message = standard_messages::request_name(
            "org.bluebus".to_owned(),
            standard_messages::DBUS_NAME_FLAG_REPLACE_EXISTING,
        );
        let response_serial = connection.send_message(&mut message, self.timeout)?;
        let msg = connection
            .wait_response(response_serial, self.timeout)?
            .unmarshall_all()
            .unwrap();
        dbg!(msg);

        let mut message = register_agent("/bluebus/agent", "KeyboardDisplay").unwrap();
        let response_serial = connection.send_message(&mut message, self.timeout).unwrap();
        let msg = connection
            .wait_response(response_serial, self.timeout)
            .unwrap()
            .unmarshall_all()
            .unwrap();
        dbg!(msg);

        let BleBuilder {
            adapter_numb,
            timeout,
        } = self;

        Ok(Ble {
            connection,
            adapter_numb,
            timeout,
        })
    }
}

pub struct Ble {
    //adapter
    connection: RpcConn,
    adapter_numb: u8,
    timeout: Timeout,
}

impl Ble {
    #[allow(dead_code)]
    pub fn connect(&mut self, adress: impl Into<String>) -> Result<(), Error> {
        let adress = adress.into().replace(":", "_");

        let mut connect = MessageBuilder::new()
            .call("Connect".into())
            .at("org.bluez".into())
            .on(format!(
                "/org/bluez/hci{}/dev_{}",
                self.adapter_numb, adress
            ))
            .with_interface("org.bluez.Device1".into()) //is always Device1
            .build();

        let response_serial = self.connection.send_message(&mut connect, self.timeout)?;
        let msg = self
            .connection
            .wait_response(response_serial, self.timeout)
            .map_err(|_| Error::CouldNotConnectToDevice)?;

        match msg.typ {
            rustbus::MessageType::Reply => Ok(()),
            rustbus::MessageType::Error => Err(Error::from(msg)),
            _ => {
                let dbg_str = format!(
                    "Unexpected Dbus message, Connect should only 
                    be awnserd with Error or Reply however we got: {:?}",
                    &msg
                );
                dbg!(&dbg_str);
                panic!();
            }
        }
    }

    fn awnser_passkey(&mut self, messg: MarshalledMessage, get_key: impl Fn() -> u32) {
        dbg!();
        let passkey: u32 = get_key();
        dbg!(passkey);
        let mut response = messg.unmarshall_all().unwrap().make_response();
        response.body.push_param(passkey).unwrap();
        dbg!(&response);
        self.connection
            .send_message(&mut response, self.timeout)
            .unwrap();
    }

    #[allow(dead_code)]
    pub fn pair(
        &mut self,
        adress: impl Into<String>,
        get_key: impl Fn() -> u32,
        timeout: Duration,
    ) -> Result<(), Error> {
        let adress = adress.into().replace(":", "_");

        let mut connect = MessageBuilder::new()
            .call("Pair".into())
            .at("org.bluez".into())
            .on(format!(
                "/org/bluez/hci{}/dev_{}",
                self.adapter_numb, adress
            ))
            .with_interface("org.bluez.Device1".into()) //is always Device1
            .build();

        let response_serial = self
            .connection
            .send_message(&mut connect, self.timeout)
            .unwrap();
        dbg!();

        let now = Instant::now();
        loop {
            let messg = self
                .connection
                .wait_call(Timeout::Duration(timeout))
                .unwrap();
            if messg.dynheader.member == Some("RequestPasskey".into()) {
                self.awnser_passkey(messg, get_key);
                break;
            }
            if now.elapsed() > timeout {
                break;
            }
        }

        let msg = self
            .connection
            .wait_response(response_serial, self.timeout)
            .unwrap();

        match msg.typ {
            rustbus::MessageType::Reply => Ok(()),
            rustbus::MessageType::Error => Err(Error::from(msg)),
            _ => {
                let dbg_str = format!(
                    "Unexpected Dbus message, Pair should only 
                    be awnserd with Error or Reply however we got: {:?}",
                    &msg
                );
                dbg!(&dbg_str);
                panic!();
            }
        }
    }

    #[allow(dead_code)]
    pub fn disconnect(&mut self, adress: impl Into<String>) -> Result<(), Error> {
        let adress = adress.into().replace(":", "_");

        let mut connect = MessageBuilder::new()
            .call("Disconnect".into())
            .at("org.bluez".into())
            .on(format!(
                "/org/bluez/hci{}/dev_{}",
                self.adapter_numb, adress
            ))
            .with_interface("org.bluez.Device1".into()) //is always Device1
            .build();

        let response_serial = self.connection.send_message(&mut connect, self.timeout)?;
        let msg = self
            .connection
            .wait_response(response_serial, self.timeout)?;

        match msg.typ {
            rustbus::MessageType::Reply => Ok(()),
            rustbus::MessageType::Error => Err(Error::from(msg)),
            _ => {
                let dbg_str = format!(
                    "Connect can only be awnserd 
                    with Error or Reply however we got: {:?}",
                    &msg
                );
                dbg!(&dbg_str);
                panic!();
            }
        }
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, adress: impl Into<String>) -> Result<(), Error> {
        let object_path = format!("/org/bluez/hci0/dev_{}", adress.into().replace(":", "_"));
        let object_path = ObjectPath::new(&object_path).unwrap();
        let mut remove = MessageBuilder::new()
            .call("RemoveDevice".into())
            .at("org.bluez".into())
            .on(format!("/org/bluez/hci{}", self.adapter_numb))
            .with_interface("org.bluez.Adapter1".into()) //is always Device1
            .build();
        remove.body.push_param(object_path)?;

        let response_serial = self.connection.send_message(&mut remove, self.timeout)?;
        let msg = self
            .connection
            .wait_response(response_serial, self.timeout)?;

        match msg.typ {
            rustbus::MessageType::Reply => Ok(()),
            rustbus::MessageType::Error => Err(Error::from(msg)),
            _ => {
                let dbg_str = format!(
                    "Remove can only be awnserd 
                    with Error or Reply however we got: {:?}",
                    &msg
                );
                dbg!(&dbg_str);
                panic!();
            }
        }
    }

    #[allow(dead_code)]
    pub fn start_discovery(&mut self) -> Result<(), Error> {
        let mut remove = MessageBuilder::new()
            .call("StartDiscovery".into())
            .at("org.bluez".into())
            .on(format!("/org/bluez/hci{}", self.adapter_numb))
            .with_interface("org.bluez.Adapter1".into()) //is always Device1
            .build();

        let response_serial = self.connection.send_message(&mut remove, self.timeout)?;
        let msg = self
            .connection
            .wait_response(response_serial, self.timeout)?;

        match msg.typ {
            rustbus::MessageType::Reply => Ok(()),
            rustbus::MessageType::Error => Err(Error::from(msg)),
            _ => {
                let dbg_str = format!(
                    "StartDiscovery can only be awnserd 
                    with Error or Reply however we got: {:?}",
                    &msg
                );
                dbg!(&dbg_str);
                panic!();
            }
        }
    }

    #[allow(dead_code)]
    pub fn stop_discovery(&mut self) -> Result<(), Error> {
        let mut remove = MessageBuilder::new()
            .call("StopDiscovery".into())
            .at("org.bluez".into())
            .on(format!("/org/bluez/hci{}", self.adapter_numb))
            .with_interface("org.bluez.Adapter1".into()) //is always Device1
            .build();

        let response_serial = self.connection.send_message(&mut remove, self.timeout)?;
        let msg = self
            .connection
            .wait_response(response_serial, self.timeout)?;

        match msg.typ {
            rustbus::MessageType::Reply => Ok(()),
            rustbus::MessageType::Error => Err(Error::from(msg)),
            _ => {
                let dbg_str = format!(
                    "StopDiscovery can only be awnserd 
                    with Error or Reply however we got: {:?}",
                    &msg
                );
                dbg!(&dbg_str);
                panic!();
            }
        }
    }

    #[allow(dead_code)]
    pub fn is_connected(&mut self, adress: impl Into<String>) -> Result<bool, Error> {
        let adress = adress.into().replace(":", "_");
        let mut is_connected = MessageBuilder::new()
            .call("Get".into())
            .at("org.bluez".into())
            .on(format!(
                "/org/bluez/hci{}/dev_{}",
                self.adapter_numb, &adress
            ))
            .with_interface("org.freedesktop.DBus.Properties".into())
            .build();
        is_connected.body.push_param("org.bluez.Device1")?;
        is_connected.body.push_param("Connected")?;

        let response_serial = self
            .connection
            .send_message(&mut is_connected, self.timeout)?;
        let mut reply = self
            .connection
            .wait_response(response_serial, self.timeout)?
            .unmarshall_all()?;

        let param = reply.params.pop().unwrap();
        let container = unwrap_container(param).unwrap();
        let variant = unwrap_variant(container).unwrap();
        let param = variant.value;
        let base = unwrap_base(param).unwrap();
        let connected = unwrap_bool(base).unwrap();

        Ok(connected)
    }

    #[allow(dead_code)]
    pub fn read(
        &mut self,
        adress: impl Into<String>,
        uuid: impl AsRef<str>,
    ) -> Result<Vec<u8>, Error> {
        let char_path = self
            .path_for_char(adress, uuid)?
            .ok_or(Error::CharacteristicNotFound)?;

        let mut read = MessageBuilder::new()
            .call("ReadValue".into())
            .at("org.bluez".into())
            .on(char_path)
            .with_interface("org.bluez.GattCharacteristic1".into()) //is always GattCharacteristic1
            .build();

        let param = empty_options_param();
        read.body.push_old_param(&param)?;

        let response_serial = self.connection.send_message(&mut read, self.timeout)?;
        let reply = self
            .connection
            .wait_response(response_serial, self.timeout)?
            .unmarshall_all()?;

        match &reply.typ {
            rustbus::MessageType::Error => return Err(to_error(reply)),
            rustbus::MessageType::Reply => (),
            _ => return Err(Error::UnexpectedDbusReply),
        }

        let mut params = reply.params;
        let param = params.pop().ok_or(Error::UnexpectedDbusReply)?;
        let container = unwrap_container(param).ok_or(Error::UnexpectedDbusReply)?;
        let array = unwrap_array(container).ok_or(Error::UnexpectedDbusReply)?;

        let data: Vec<u8> = array
            .values
            .into_iter()
            .map(|param| param.into_byte())
            .collect::<Result<Vec<u8>, _>>()
            .map_err(|_| Error::UnexpectedDbusReply)?;
        Ok(data)
    }

    #[allow(dead_code)]
    pub fn write(
        &mut self,
        adress: impl Into<String>,
        uuid: impl AsRef<str>,
        data: impl AsRef<[u8]>,
    ) -> Result<(), Error> {
        let char_path = self
            .path_for_char(adress, uuid)?
            .ok_or(Error::CharacteristicNotFound)?;

        let mut write = MessageBuilder::new()
            .call("WriteValue".into())
            .at("org.bluez".into())
            .on(char_path)
            .with_interface("org.bluez.GattCharacteristic1".into()) //is always GattCharacteristic1
            .build();

        let options = empty_options_param();
        write.body.push_param(data.as_ref())?;
        write.body.push_old_param(&options)?;

        let response_serial = self.connection.send_message(&mut write, self.timeout)?;
        let reply = self
            .connection
            .wait_response(response_serial, self.timeout)?
            .unmarshall_all()?;

        match &reply.typ {
            rustbus::MessageType::Error => {
                dbg!(&reply);
                return Err(to_error(reply));
            }
            rustbus::MessageType::Reply => (),
            _ => return Err(Error::UnexpectedDbusReply),
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn notify(
        &mut self,
        adress: impl Into<String>,
        uuid: impl AsRef<str>,
    ) -> Result<File, Error> {
        let char_path = self
            .path_for_char(adress, uuid)?
            .ok_or(Error::CharacteristicNotFound)?;

        let mut aquire_notify = MessageBuilder::new()
            .call("AcquireNotify".into())
            .at("org.bluez".into())
            .on(char_path)
            .with_interface("org.bluez.GattCharacteristic1".into()) //is always GattCharacteristic1
            .build();

        let param = empty_options_param();
        aquire_notify.body.push_old_param(&param)?;
        dbg!(&aquire_notify);

        let response_serial = self
            .connection
            .send_message(&mut aquire_notify, self.timeout)?;
        let reply = self
            .connection
            .wait_response(response_serial, self.timeout)?
            .unmarshall_all()?;
        dbg!(&reply);

        match &reply.typ {
            rustbus::MessageType::Error => return Err(to_error(reply)),
            rustbus::MessageType::Reply => (),
            _ => return Err(Error::UnexpectedDbusReply),
        }

        let message::Message {
            mut params,
            mut raw_fds,
            ..
        } = reply;
        let mtu = params.pop().ok_or(Error::UnexpectedDbusReply)?;
        let mtu = unwrap_base(mtu).ok_or(Error::UnexpectedDbusReply)?;
        let mtu = unwrap_u16(mtu).ok_or(Error::UnexpectedDbusReply)?;
        dbg!(mtu);

        dbg!(params);
        let raw_fd = raw_fds.pop().ok_or(Error::NoFdReturned)?;
        dbg!(&raw_fd);
        let file = unsafe { File::from_raw_fd(raw_fd) };
        Ok(file)
    }

    fn path_for_char(
        &mut self,
        adress: impl Into<String>,
        char_uuid: impl AsRef<str>,
    ) -> Result<Option<String>, Error> {
        let mut get_paths = MessageBuilder::new()
            .call("GetManagedObjects".into())
            .at("org.bluez".into())
            .on("/".into())
            .with_interface("org.freedesktop.DBus.ObjectManager".into())
            .build();

        let response_serial = self.connection.send_message(&mut get_paths, self.timeout)?;
        let mut reply = self
            .connection
            .wait_response(response_serial, self.timeout)?
            .unmarshall_all()?;

        let param = reply.params.pop().unwrap();
        let container = unwrap_container(param).unwrap();
        let dict = unwrap_dict(container).unwrap();

        let device_path = format!(
            "/org/bluez/hci{}/dev_{}",
            self.adapter_numb,
            adress.into().replace(":", "_")
        );

        for (path, base) in dict
            .into_iter()
            .filter_map(unwrap_objectpath)
            .filter(|(p, _)| p.contains(&device_path))
            .filter(|(p, _)| p.contains("char"))
            .filter(|(p, _)| !p.contains("desc"))
        {
            let container = unwrap_container(base).unwrap();
            let mut dict = unwrap_dict(container).unwrap();
            let gatt_char = dict
                .remove(&params::Base::String(
                    "org.bluez.GattCharacteristic1".into(),
                ))
                .expect("char object path should always have GattCharacteristic1");
            let gatt_char = unwrap_container(gatt_char).unwrap();
            let mut gatt_char = unwrap_dict(gatt_char).unwrap();
            let uuid = gatt_char
                .remove(&params::Base::String("UUID".into()))
                .expect("char object should always have a UUID");
            let uuid = unwrap_container(uuid).unwrap();
            let uuid = unwrap_variant(uuid).unwrap();
            let uuid = uuid.value;
            let uuid = unwrap_base(uuid).unwrap();
            let uuid = unwrap_string(uuid).unwrap();

            if uuid == char_uuid.as_ref() {
                return Ok(Some(path));
            }
        }
        Ok(None)
    }
}

fn empty_options_param<'a, 'e>() -> rustbus::params::Param<'a, 'e> {
    let dic = params::Dict {
        key_sig: rustbus::signature::Base::String,
        value_sig: rustbus::signature::Type::Container(rustbus::signature::Container::Variant),
        map: HashMap::new(),
    };
    let dic = rustbus::params::Container::Dict(dic);
    rustbus::params::Param::Container(dic)
}
