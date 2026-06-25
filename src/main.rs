use hidapi::{HidApi, HidDevice};
use anyhow::anyhow;
use time::OffsetDateTime;

const VID: u16 = 0x05ac;
const PID: u16 = 0x024f;
const PRODUCT_NAME: &'static str = "AK650";
const INTERFACE_NUMBER: i32 = 3;

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
        let time = OffsetDateTime::now_local()?;
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
        write_buf.fill(0);
        write_buf[3] = 0x5A; // command byte
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
    // init hidapi
    let hidapi = HidApi::new()?;

    let device = search(&hidapi)?
        .ok_or(anyhow!("No device found"))?;

    send_time_payload(&device)?;

    println!("Done");

    Ok(())
}

fn search(hidapi: &HidApi) -> anyhow::Result<Option<HidDevice>> {
    println!("Looking for {PRODUCT_NAME} on interface number {INTERFACE_NUMBER}");
    for device in hidapi.device_list() {
        let vid = device.vendor_id();
        let pid = device.product_id();
        let interface_number = device.interface_number();
        let product_name = device.product_string().unwrap_or("");

        if vid == VID && pid == PID && interface_number == INTERFACE_NUMBER && product_name == PRODUCT_NAME {
            println!("Found {PRODUCT_NAME}");
            let handle = device.open_device(hidapi)?;
            return Ok(Some(handle));
        }
    }
    Ok(None)
}

fn send_time_payload(device: &HidDevice) -> anyhow::Result<()> {
    device.set_blocking_mode(true)?;

    let mut read_buf = vec![0_u8; 65];
    let mut write_buf = vec![0_u8; 65];

    // init packet 1
    write_buf[1] = 0x04;
    write_buf[2] = 0x18;
    device.send_feature_report(&write_buf)?;
    device.get_feature_report(&mut read_buf)?;

    // init packet 2
    write_buf.fill(0);
    write_buf[1] = 0x04;
    write_buf[2] = 0x28;
    write_buf[9] = 0x01;
    device.send_feature_report(&write_buf)?;
    device.get_feature_report(&mut read_buf)?;

    // time sync packet
    // let payload = TimePayload {
    //     year: 0,
    //     month: 0,
    //     day: 0,
    //     hour: 0,
    //     minute: 0,
    //     second: 0,
    //     day_of_week: 0,
    // };
    TimePayload::from_current_time()?.generate_payload(&mut write_buf);
    device.send_feature_report(&write_buf)?;
    device.get_feature_report(&mut read_buf)?;

    // epilogue
    write_buf.fill(0);
    write_buf[1] = 0x04;
    write_buf[2] = 0x02;
    device.send_feature_report(&write_buf)?;
    device.get_feature_report(&mut read_buf)?;

    Ok(())
}