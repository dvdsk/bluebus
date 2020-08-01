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
    AuthenticationFailed,
    UnknownErrorMessage(String),
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
        to_error(msg.unmarshall_all().unwrap())
    }
}

impl From<rustbus::Error> for Error {
    fn from(err: rustbus::Error) -> Error {
        Error::RustbusError(err)
    }
}

/*pub enum ErrorContext {
    AquireNotify(String),
    ReadValue(String),
    WriteValue(String),
}*/

fn unpack_msg(msg: &mut Message) -> Option<String> {
    let error_msg = msg.params.pop()?.into_string().ok()?;
    Some(error_msg)
}

pub fn to_error(mut msg: Message) -> Error {
    if let Some(error_msg) = unpack_msg(&mut msg) {
        match error_msg.as_str() {
            "Operation is not supported" => return Error::OperationNotSupported,
            "Invalid Length" => return Error::InvalidLength,
            _ => (),
        }
    } 
    if let Some(error_name) = &msg.dynheader.error_name {
        match error_name.as_str() {
            "org.bluez.Error.AuthenticationFailed" => return Error::AuthenticationFailed,
            _ => (),
        }
    }

    Error::UnknownErrorMessage(format!("{:?}", msg))
}