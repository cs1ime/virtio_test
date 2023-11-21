#![no_std]


pub enum VirtioDeviceType {
    None = 0,
    Net = 1,
    Block = 2,
}

pub struct VirtMmioRegs {
    magic: u32,
    version: u32,
    device_id: u32,
    vendor_id: u32,
    q_num_max: u32,
}

impl VirtMmioRegs {
    pub fn default() -> Self
    {
        Self {
            magic: 0,
            version: 0,
            device_id: 0,
            vendor_id: 0,
            q_num_max: 0,
        }
    }

    pub fn init(&mut self, id: VirtioDeviceType) {
        self.magic = 0x74726976;
        self.version = 0x2;
        self.vendor_id = 0x8888;
        self.device_id = id as u32;
        self.q_sel = 0;
    }
}

pub struct VirtMmio 
{
    inner: Arc<Mutex<VirtMmioInner>>
}

struct VirtioMmioInner {
    id: usize,
    regs: VirtMmioRegs,
}


