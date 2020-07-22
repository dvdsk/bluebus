use rustbus::params::message::Message;

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
    CouldNotRemoveCache(std::io::Error),
    OperationNotSupported,
    InvalidLength,
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::CouldNotRemoveCache(err)
    }
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
    ReadValue(String),
    WriteValue(String),
}

fn unpack_msg<'a,'e>(mut msg: Message<'a,'e>) -> Option<String> {
    let error_msg = msg.params.pop()?.into_string().ok()?;
    Some(error_msg)

}


pub fn to_error<'a,'e>(msg: Message<'a,'e>, _: ErrorContext) -> Error {
    if let Some(error_msg) = unpack_msg(msg) {
        match error_msg.as_str() {
            "Operation is not supported" => Error::OperationNotSupported,
            "Invalid Length" => Error::InvalidLength,
            _ => Error::CharacteristicNotFound,
        }
    } else {
        Error::CharacteristicNotFound
    }
}