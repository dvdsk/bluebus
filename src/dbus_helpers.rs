use rustbus;
use rustbus::{MessageBuilder, params, params::Param, message_builder::MarshalledMessage};
use crate::error::Error;

pub fn unwrap_variant<'e, 'a>(container: params::Container<'e, 'a>)
-> Option<Box<rustbus::params::Variant<'a, 'e>>> {
   match container {
        params::Container::Variant(v) => Some(v),
        _ => None,
   }
}

pub fn unwrap_string(base: params::Base)
 -> Option<String> {
    match base {
        params::Base::String(s) => Some(s),
        _ => None,
    }
} 

pub fn unwrap_bool(base: params::Base)
 -> Option<bool> {
    match base {
        params::Base::Boolean(b) => Some(b),
        _ => None,
    }
} 

pub fn unwrap_objectpath<'a, 'e>(tup: (params::Base, params::Param<'a, 'e>))
 -> Option<(String,params::Param<'a, 'e>)> { 
    let (base, param) = tup;
    match base {
        params::Base::ObjectPath(s) => Some((s,param)),
        _ => None,
    }
}

pub fn unwrap_dict<'e, 'a>(param: params::Container<'e, 'a>)
 -> Option<rustbus::params::DictMap<'a, 'e>> {
    match param {
        params::Container::Dict(c) => Some(c.map),
        _ => None,
    }
}

pub fn unwrap_container<'a, 'e>(param: params::Param<'a, 'e>)
 -> Option<params::Container<'a, 'e>> {
    match param {
        params::Param::Container(c) => Some(c),
        _ => None,
    }
}

pub fn unwrap_base<'a, 'e>(param: params::Param<'a, 'e>)
 -> Option<params::Base<'a>> {
    match param {
        params::Param::Base(b) => Some(b),
        _ => None,
    }
}

pub fn get_name_owner(name: String)
     -> Result<MarshalledMessage, Error> {
    
    let mut msg = MessageBuilder::new()
        .call("GetNameOwner".into())
        .on("/org/freedesktop/DBus".into())
        .with_interface("org.freedesktop.DBus".into())
        .at("org.freedesktop.DBus".into())
        .build();

    msg.body.push_param(name)?;
    Ok(msg)
}

pub fn register_agent(obj_path: &str, capability: &str)
     -> Result<MarshalledMessage, Error> {
    
    let param1 = Param::Base(params::Base::ObjectPath(obj_path.to_owned()));
    let param2 = Param::Base(params::Base::String(capability.to_owned()));

    let mut msg = MessageBuilder::new()
        .call("RegisterAgent".into())
        .on("/org/bluez".into())
        .with_interface("org.bluez.AgentManager1".into())
        .at("org.bluez".into())
        .build();

    msg.body.push_old_params(&vec![param1, param2])?;
    Ok(msg)
}