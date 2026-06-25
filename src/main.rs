use hidapi::{DeviceInfo, HidApi, HidDevice};
use tracing::{info, debug, warn, error};
use anyhow::anyhow;

const VID: u16 = 0x05ac;
const PID: u16 = 0x024f;
const PRODUCT_NAME: &'static str = "AK650";
const INTERFACE_NUMBER: i32 = 3;

#[repr(C)]
struct DriverPayload {


}

fn main() -> anyhow::Result<()> {
    // init logger
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;

    // init hidapi
    let hidapi = HidApi::new()?;

    let device = search(&hidapi)?
        .ok_or(anyhow!("No device found"))?;

    send_time_payload(&device)?;

    info!("Done");

    Ok(())
}

fn search(hidapi: &HidApi) -> anyhow::Result<Option<HidDevice>> {
    info!("Looking for {PRODUCT_NAME} on interface number {INTERFACE_NUMBER}");
    for device in hidapi.device_list() {
        let vid = device.vendor_id();
        let pid = device.product_id();
        let interface_number = device.interface_number();
        let product_name = device.product_string().unwrap_or("");

        if vid == VID && pid == PID && interface_number == INTERFACE_NUMBER && product_name == PRODUCT_NAME {
            info!("Found {PRODUCT_NAME}");
            let handle = device.open_device(hidapi)?;
            return Ok(Some(handle));
        }
    }
    Ok(None)
}

fn send_time_payload(device: &HidDevice) -> anyhow::Result<()> {
    let mut payload = vec![0_u8; 64];

    payload[0] = 0x04;
    payload[1] = 0x18;
    device.send_output_report(&payload)?;

    payload = vec![0_u8; 64];
    payload[0] = 0x04;
    payload[1] = 0x28;
    payload[8] = 0x01;
    device.send_output_report(&payload)?;

    payload = vec![0_u8; 64];
    payload[2] = 0x5A;
    // TODO: time data

    // magic footer
    payload[62] = 0xAA;
    payload[63] = 0x55;
    device.send_output_report(&payload)?;

    payload = vec![0_u8; 64];
    payload[0] = 0x04;
    payload[1] = 0x02;
    device.send_output_report(&payload)?;


    Ok(())
}