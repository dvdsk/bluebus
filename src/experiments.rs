use rustbus::{get_system_bus_path, MessageBuilder, Conn, RpcConn, standard_messages};

pub fn exp() -> Result<(), rustbus::client_conn::Error> {
    let session_path = get_system_bus_path()?;
    let con = Conn::connect_to_bus(session_path, true)?;
    let mut rpc_con = RpcConn::new(con);
    // send the obligatory hello message
    rpc_con.send_message(&mut standard_messages::hello(), None)?;


    let mut test_msg = MessageBuilder::new()
        .call("StartDiscovery".into())
        //.call("StopDiscovery".into()) //member ?= method name
        .at("org.bluez".into()) //destination ?= bus name
        .on("/org/bluez/hci0".into()) //object path
        .with_interface("org.bluez.Adapter1".into())
        .build();

    println!("Send message: {:?}", test_msg);
    let response_serial = rpc_con.send_message(&mut test_msg, None)?;

    println!("\n");
    println!("Wait for start discovery");
    let msg = rpc_con.wait_response(response_serial, None)?;
    println!("Got response: {:?}", msg);

/////////////////////////////////////////////////////////////////////////

    let mut test_msg = MessageBuilder::new()
    .call("Connect".into())
    //.call("StopDiscovery".into()) //member ?= method name
    .at("org.bluez".into()) //destination ?= bus name
    .on("/org/bluez/hci0/dev_C6_46_56_AC_2C_4C".into()) //object path
    .with_interface("org.bluez.Device1".into())
    .build();

    println!("Send message: {:?}", test_msg);
    let response_serial = rpc_con.send_message(&mut test_msg, None)?;

    println!("\n");
    println!("Wait for connect");
    let msg = rpc_con.wait_response(response_serial, None)?;
    println!("Got response: {:?}", msg);

    Ok(())
}