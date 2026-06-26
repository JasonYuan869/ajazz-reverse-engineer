use anyhow::Context;
use env_logger::{Builder, Env, Target};
use hidapi::{HidApi, HidDevice};
use log::info;
use time::OffsetDateTime;

const VID: u16 = 0x05ac; // NOTE: this is Apple's USB vendor ID; confirmed via packet sniff.
const PID: u16 = 0x024f;
const PRODUCT_NAME: &str = "AK650";
const INTERFACE_NUMBER: i32 = 3;

/// HID feature-report buffer length. Index 0 is the report ID; bytes 1..=64
/// carry the payload, so the buffer is 65 bytes wide.
const REPORT_LEN: usize = 65;

/// Command byte that marks a time-sync packet.
const CMD_TIME: u8 = 0x5A;

struct TimePayload {
    year: u8, // years since 2000
    month: u8, // jan = 1, dec = 12
    day: u8, // 1 indexed
    hour: u8, // 24-hour time
    minute: u8,
    second: u8,
    day_of_week: u8, // 0 is Sunday
}

impl TimePayload {
    fn from_current_time() -> anyhow::Result<Self> {
        let time = OffsetDateTime::now_local()
            .context("could not determine local time")?;
        let payload = TimePayload {
            year: (time.year() - 2000) as u8,
            month: time.month().into(),
            day: time.day(),
            hour: time.hour(),
            minute: time.minute(),
            second: time.second(),
            day_of_week: time.weekday().number_days_from_sunday(),
        };
        Ok(payload)
    }

    fn generate_payload(&self, write_buf: &mut [u8]) {
        debug_assert_eq!(write_buf.len(), REPORT_LEN);
        write_buf.fill(0);
        write_buf[3] = CMD_TIME; // command byte
        write_buf[4] = self.year;
        write_buf[5] = self.month;
        write_buf[6] = self.day;
        write_buf[7] = self.hour;
        write_buf[8] = self.minute;
        write_buf[9] = self.second;
        // padding
        write_buf[11] = self.day_of_week;
        // padding
        write_buf[63] = 0xAA; // magic footer
        write_buf[64] = 0x55; // magic footer
    }
}

fn main() -> anyhow::Result<()> {
    // Init logging
    Builder::from_env(Env::default().default_filter_or("info"))
        .target(Target::Stdout)
        .init();

    // init hidapi
    let hidapi = HidApi::new().context("failed to initialize hidapi")?;

    let Some(device) = search(&hidapi)? else {
        info!("No device found, nothing to sync");
        return Ok(());
    };

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
            let handle = device
                .open_device(hidapi)
                .with_context(|| format!("failed to open {PRODUCT_NAME}"))?;
            return Ok(Some(handle));
        }
    }
    Ok(None)
}

/// Sends one feature report and reads the device's response back.
fn exchange(device: &HidDevice, write_buf: &[u8], read_buf: &mut [u8]) -> anyhow::Result<()> {
    device
        .send_feature_report(write_buf)
        .context("send_feature_report failed")?;
    device
        .get_feature_report(read_buf)
        .context("get_feature_report failed")?;
    Ok(())
}

fn send_time_payload(device: &HidDevice) -> anyhow::Result<()> {
    device
        .set_blocking_mode(true)
        .context("failed to set blocking mode")?;

    let mut read_buf = vec![0_u8; REPORT_LEN];
    let mut write_buf = vec![0_u8; REPORT_LEN];
    info!("Sending initialization packets");

    // Wake up device
    write_buf.fill(0);
    write_buf[1] = 0x04;
    write_buf[2] = 0x28;
    write_buf[9] = 0x01;
    exchange(device, &write_buf, &mut read_buf)?;

    // Sync time
    TimePayload::from_current_time()?.generate_payload(&mut write_buf);

    info!("Sending time sync packet");
    exchange(device, &write_buf, &mut read_buf)?;

    Ok(())
}
