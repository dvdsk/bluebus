use std::collections::HashMap;
use std::os::unix::io::RawFd;

pub use rustbus::client_conn::Timeout;
use rustbus::params::message;
use rustbus::{params, MessageBuilder};

use crate::error::{Context, Error};
use crate::dbus_helpers::*;
use crate::Ble;

impl Ble {
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
            .on(char_path.clone())
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
            rustbus::MessageType::Error => return Err(Error::from((reply, Context::ReadValue(char_path)))),
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
            .on(char_path.clone())
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
                return Err(Error::from((reply, Context::WriteValue(char_path))));
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
    ) -> Result<RawFd, Error> {
        let char_path = self
            .path_for_char(adress, uuid)?
            .ok_or(Error::CharacteristicNotFound)?;

        let mut aquire_notify = MessageBuilder::new()
            .call("AcquireNotify".into())
            .at("org.bluez".into())
            .on(char_path.clone())
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
            rustbus::MessageType::Error 
                => return Err(Error::from((reply, Context::AquireNotify(char_path)))),
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
        let fd = raw_fds.pop().ok_or(Error::NoFdReturned)?;
        //let file = unsafe { File::from_raw_fd(raw_fd) };
        Ok(fd)
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
