#[repr(packed)]
#[allow(non_snake_case)]
pub struct DeviceDescriptor {
    pub bLength: u8,
    pub bDescriptorType: u8,
    pub bcdUSB: u16,
    pub bDeviceClass: u8,
    pub bDeviceSubClass: u8,
    pub bDeviceProtocol: u8,
    pub bMaxPacketSize0: u8,
    pub idVendor: u16,
    pub idProduct: u16,
    pub bcdDevice: u16,
    pub iManufacturer: u8,
    pub iProduct: u8,
    pub iSerialNumber: u8,
    pub bNumConfigurations: u8,
}

#[repr(packed)]
#[allow(non_snake_case)]
pub struct ConfigDescriptor {
    pub bLength: u8,
    pub bDescriptorType: u8,
    pub wTotalLength: u16,
    pub bNumInterfaces: u8,
    pub bConfigurationValue: u8,
    pub iConfiguration: u8,
    pub bmAttributes: u8,
    pub bMaxPower: u8,
}

#[repr(packed)]
#[allow(non_snake_case)]
pub struct InterfaceDescriptor {
    pub bLength: u8,
    pub bDescriptorType: u8,
    pub bInterfaceNumber: u8,
    pub bAlternateSetting: u8,
    pub bNumEndpoints: u8,
    pub bInterfaceClass: u8,
    pub bInterfaceSubClass: u8,
    pub bInterfaceProtocol: u8,
    pub iInterface: u8,
}

#[repr(packed)]
#[allow(non_snake_case)]
pub struct EndpointDescriptor {
    pub bLength: u8,
    pub bDescriptorType: u8,
    pub bEndpointAddress: u8,
    pub bmAttributes: u8,
    pub wMaxPacketSize: u16,
    pub bInterval: u8,
}

#[repr(packed)]
#[allow(non_snake_case)]
pub struct HidDescriptor {
    pub bLength: u8,
    pub bDescriptorType: u8,
    pub bcdHID: u16,
    pub bCountryCode: u8,
    pub bNumDescriptors: u8,
}

#[repr(packed)]
#[allow(non_snake_case)]
pub struct HidReport {
    pub bReportDescriptorType: u8,
    pub wDescriptorLength: u16,
}

#[repr(packed)]
#[allow(non_snake_case)]
pub struct HidFunction {
    pub hid_descriptor: HidDescriptor,
    pub hid_report: HidReport,
}

pub const STRING_DESCR0: &[u8] = &[0x04, 0x03, 0x09, 0x04];

pub fn build_string_descr(buf: &mut [u8], data: &str) -> Option<usize> {
    let utf16 = data.encode_utf16();

    let iter = buf[2..].chunks_exact_mut(2).zip(utf16).enumerate().map(|(idx, (dst, chr))| {
        dst.copy_from_slice(&chr.to_le_bytes());
        idx
    });
    iter.last().map(|idx| {
        let len = (idx + 1) * 2 + 2;
        buf[0..2].copy_from_slice(&[len as u8, 0x03]);
        len
    })
}
