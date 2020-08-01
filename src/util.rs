use crate::dbus_helpers::{unwrap_container, unwrap_variant};
use crate::error::Error;
use crate::{Ble, TIMEOUT};
use rustbus::MessageBuilder;
use std::fs::remove_file;
use std::path::PathBuf;

impl Ble {
    fn adapter_adress(&mut self) -> Result<String, Error> {
        let mut get_addr = MessageBuilder::new()
            .call("Get".into())
            .on("/org/bluez/hci0".into())
            .with_interface("org.freedesktop.DBus.Properties".into())
            .at("org.bluez".into())
            .build();

        let adapter = "org.bluez.Adapter1".to_string();
        get_addr.body.push_param2(adapter, "Address")?;

        let response_serial = self.connection.send_message(&mut get_addr, TIMEOUT)?;
        let mut reply = self
            .connection
            .wait_response(response_serial, TIMEOUT)?
            .unmarshall_all()?;
        let param = reply.params.pop().unwrap();
        let p = unwrap_container(param).unwrap();
        let p = unwrap_variant(p).unwrap();
        let adress = p.value.into_string().unwrap();

        Ok(adress)
    }

    /// util function that clears the device cache. When used after removing
    /// the device from bluez this well make sure all caracteristics are rediscovered
    /// if the device is added again (by connecting). This function will need to run
    /// with superuser privileges.
    pub fn remove_attribute_cache(&mut self, device_mac: &str) -> Result<(), Error> {
        let mut path = PathBuf::from("/var/lib/bluetooth");
        path.push(self.adapter_adress()?);
        path.push("cache");
        path.push(device_mac);

        remove_file(path)?;
        Ok(())
    }
}
