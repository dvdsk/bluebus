use rustbus::params::message::Message;
use super::BleBuilder;

#[derive(Debug)]
pub enum Error {
    RustbusError(rustbus::Error),
    DbusConnectionError(rustbus::client_conn::Error),
    DBusUnMashallError(rustbus::wire::unmarshal::Error),
    CouldNotConnectToDevice,
    CouldNotConnectToBus(String),
    UuidNotFound,
    DeviceNotFound,
    CharacteristicNotFound,
    CharacteristicNotFoundCauseUnconnected,
    NoFdReturned,
    UnexpectedDbusReply,
}

impl From<rustbus::wire::unmarshal::Error> for Error {
    fn from(err: rustbus::wire::unmarshal::Error) -> Error {
        Error::DBusUnMashallError(err)
    }
}

impl From<rustbus::client_conn::Error> for Error {
    fn from(err: rustbus::client_conn::Error) -> Error {
        Error::DbusConnectionError(err)
    }
}
// //TODO differentiate timeout here
impl From<rustbus::message_builder::MarshalledMessage> for Error {
    fn from(msg: rustbus::message_builder::MarshalledMessage) -> Error {
        Error::CouldNotConnectToBus(format!("{:?}",msg.unmarshall_all()))
    }
}

impl From<rustbus::Error> for Error {
    fn from(err: rustbus::Error) -> Error {
        Error::RustbusError(err)
    }
}

pub enum ErrorContext {
    AquireNotify(String),
}

/*fn ble_is_connected(object_path: String) -> Result<bool, Error> {
    let adress = object_path.split("dev_").nth(0)
        .expect("malformed object path");
    let adress = (&adress[..16]).to_owned().replace(":","_");

    let mut ble = BleBuilder::new().unwrap().build().unwrap();
    ble.is_connected(adress)
}*/

pub fn to_error<'a,'e>(msg: Message<'a,'e>, context: ErrorContext) -> Error {
    Error::CharacteristicNotFound
    /*match context {
        ErrorContext::AquireNotify(object_path) => {
            if msg.error_name == Some("org.freedesktop.DBus.Error.UnknownMethod".to_owned()) {
                if !ble_is_connected(object_path).unwrap_or(true) {
                    Error::CharacteristicNotFoundCauseUnconnected
                } else {
                    Error::CharacteristicNotFound
                }
            } else {
                Error::UnexpectedDbusReply
            }
        }
    }*/
}