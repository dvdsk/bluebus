//use std::fs::File;
use std::time::{Duration, Instant};

pub use rustbus::client_conn::Timeout;
use rustbus::message_builder::MarshalledMessage;
use rustbus::wire::marshal::traits::ObjectPath;
use rustbus::MessageBuilder;

use crate::dbus_helpers::*;
use crate::error::{Context, Error};
use crate::Ble;

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
            rustbus::MessageType::Error => Err(Error::from((msg, Context::Connect))),
            _ => {
                let dbg_str = format!(
                    "Unexpected Dbus message, Connect should only 
                    be awnserd with Error or Reply however we got: {:?}",
                    &msg
                );
                panic!(dbg_str);
            }
        }
    }

    fn awnser_passkey(&mut self, messg: MarshalledMessage, get_key: impl Fn() -> u32) {
        let passkey: u32 = get_key();
        let mut response = messg.unmarshall_all().unwrap().make_response();
        response.body.push_param(passkey).unwrap();
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

        let now = Instant::now();
        loop {
            let messg = self
                .connection
                .wait_call(Timeout::Duration(timeout))
                .map_err(|e| match e {
                    rustbus::client_conn::Error::TimedOut => {
                        dbg!("test timeout");
                        Error::PairingTimeOut
                    }
                    _ => e.into(),
                })?;
            if messg.dynheader.member == Some("RequestPasskey".into()) {
                self.awnser_passkey(messg, get_key);
                break;
            }
            if now.elapsed() > timeout {
                return Err(Error::PairingTimeOut);
            }
        }

        let msg = self
            .connection
            .wait_response(response_serial, self.timeout)
            .unwrap();

        match msg.typ {
            rustbus::MessageType::Reply => Ok(()),
            rustbus::MessageType::Error => Err(Error::from((msg, Context::Pair))),
            _ => {
                let dbg_str = format!(
                    "Unexpected Dbus message, Pair should only 
                    be awnserd with Error or Reply however we got: {:?}",
                    &msg
                );
                panic!(dbg_str);
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
            rustbus::MessageType::Error => Err(Error::from((msg, Context::Disconnect))),
            _ => {
                let dbg_str = format!(
                    "Connect can only be awnserd 
                    with Error or Reply however we got: {:?}",
                    &msg
                );
                panic!(dbg_str);
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
            rustbus::MessageType::Error => Err(Error::from((msg, Context::Remove))),
            _ => {
                let dbg_str = format!(
                    "Remove can only be awnserd 
                    with Error or Reply however we got: {:?}",
                    &msg
                );
                panic!(dbg_str);
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
            rustbus::MessageType::Error => Err(Error::from((msg, Context::StartDiscovery))),
            _ => {
                let dbg_str = format!(
                    "StartDiscovery can only be awnserd 
                    with Error or Reply however we got: {:?}",
                    &msg
                );
                panic!(dbg_str);
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
            rustbus::MessageType::Error => Err(Error::from((msg, Context::StopDiscovery))),
            _ => {
                let dbg_str = format!(
                    "StopDiscovery can only be awnserd 
                    with Error or Reply however we got: {:?}",
                    &msg
                );
                panic!(dbg_str);
            }
        }
    }
}
