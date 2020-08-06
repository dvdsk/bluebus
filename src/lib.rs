use std::time::Duration;

pub use rustbus::client_conn::Timeout;
use rustbus::{get_system_bus_path, standard_messages, Conn, RpcConn};

mod dbus_helpers;
use dbus_helpers::*;

mod error;
pub use error::{Error, Context};
pub mod operations;
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

        /*let mut message = standard_messages::request_name(
            "org.bluebus".to_owned(),
            standard_messages::DBUS_NAME_FLAG_REPLACE_EXISTING,
        );
        let response_serial = connection.send_message(&mut message, self.timeout)?;
        let msg = connection
            .wait_response(response_serial, self.timeout)?
            .unmarshall_all()
            .unwrap();
        dbg!(msg);*/

        let mut message = register_agent("/bluebus/agent", "KeyboardDisplay").unwrap();
        let response_serial = connection.send_message(&mut message, self.timeout).unwrap();
        let msg = connection
            .wait_response(response_serial, self.timeout)
            .unwrap()
            .unmarshall_all()
            .unwrap();

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
