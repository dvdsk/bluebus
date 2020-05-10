use rustbus::message::Message;
use super::BleBuilder;

#[derive(Debug)]
pub enum Error {
    DbusConnectionError(rustbus::client_conn::Error),
    CouldNotConnect(String),
    UuidNotFound,
    DeviceNotFound,
    CharacteristicNotFound,
    CharacteristicNotFoundCauseUnconnected,
    NoFdReturned,
    UnexpectedDbusReply,
}

impl From<rustbus::client_conn::Error> for Error {
    fn from(err: rustbus::client_conn::Error) -> Error {
        Error::DbusConnectionError(err)
    }
}
impl<'a> From<rustbus::message::Message<'a,'a>> for Error {
    fn from(msg: rustbus::message::Message<'a,'a>) -> Error {
        Error::CouldNotConnect(format!("{:?}",msg))
    }
}

pub enum ErrorContext {
    AquireNotify(String),
}

fn ble_is_connected(object_path: String) -> Result<bool, Error> {
    let adress = object_path.split("dev_").nth(0)
        .expect("malformed object path");
    let adress = (&adress[..16]).to_owned().replace(":","_");

    let mut ble = BleBuilder::new().unwrap().build().unwrap();
    ble.is_connected(adress)
}

pub fn to_error<'a,'e>(msg: Message<'a,'e>, context: ErrorContext) -> Error {
    match context {
        ErrorContext::AquireNotify(object_path) => {
            if msg.error_name == Some("org.freedesktop.DBus.Error.UnknownMethod".to_owned()) {
                if ble_is_connected(object_path).unwrap_or(true) {
                    Error::CharacteristicNotFoundCauseUnconnected
                } else {
                    Error::CharacteristicNotFound
                }
            } else {
                Error::UnexpectedDbusReply
            }
        }
    }
}