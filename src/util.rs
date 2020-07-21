use std::fs::remove_file;
use std::path::Path;

/// util function that clears the device cache. When used after removing
/// the device from bluez this well make sure all caracteristics are rediscovered
/// if the device is added again (by connecting)
pub fn remove_attribute_cache(device_mac: &str, adapter: &str){
    let mut path = PathBuf::new("/var/lib/bluetooth");
    path.push(adapter);
    path.push("cache");
    path.push(device_mac);

    remove_file(path);
}