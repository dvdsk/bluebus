use rustbus;

pub fn unwrap_variant<'e, 'a>(container: rustbus::Container<'e, 'a>)
-> Option<Box<rustbus::params::Variant<'a, 'e>>> {
   match container {
       rustbus::Container::Variant(v) => Some(v),
       _ => None,
   }
}

pub fn unwrap_string(base: rustbus::params::Base)
 -> Option<String> {
    match base {
        rustbus::params::Base::String(s) => Some(s),
        _ => None,
    }
} 

pub fn unwrap_objectpath<'a, 'e>(tup: (rustbus::Base, rustbus::Param<'a, 'e>))
 -> Option<(String,rustbus::Param<'a, 'e>)> { 
    let (base, param) = tup;
    match base {
        rustbus::params::Base::ObjectPath(s) => Some((s,param)),
        _ => None,
    }
}

pub fn unwrap_dict<'e, 'a>(param: rustbus::Container<'e, 'a>)
 -> Option<rustbus::params::DictMap<'a, 'e>> {
    match param {
        rustbus::Container::Dict(c) => Some(c.map),
        _ => None,
    }
}

pub fn unwrap_container<'a, 'e>(param: rustbus::params::Param<'a, 'e>)
 -> Option<rustbus::Container<'a, 'e>> {
    match param {
        rustbus::params::Param::Container(c) => Some(c),
        _ => None,
    }
}

pub fn unwrap_base<'a, 'e>(param: rustbus::params::Param<'a, 'e>)
 -> Option<rustbus::Base<'a>> {
    match param {
        rustbus::params::Param::Base(b) => Some(b),
        _ => None,
    }
}