
pub const VIRTQ_READY: usize = 1;
pub const VIRTQ_DESC_F_NEXT: u16 = 1;
pub const VIRTQ_DESC_F_WRITE: u16 = 2;

pub const VRING_USED_F_NO_NOTIFY: usize = 1;

pub const DESC_QUEUE_SIZE: usize = 512;

#[repr(C, align(16))]
#[derive(Copy, Clone)]
struct VringDesc {
    /*Address (guest-physical)*/
    pub addr: usize,
    /* Length */
    len: u32,
    /* The flags as indicated above */
    flags: u16,
    /* We chain unused descriptors via this, too */
    next: u16,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct VringAvail {
    flags: u16,
    idx: u16,
    ring: [u16; 512],
}

#[repr(C)]
#[derive(Copy, Clone)]
struct VringUsedElem {
    pub id: u32,
    pub len: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct VringUsed {
    flags: u16,
    idx: u16,
    ring: [VringUsedElem; 512],
}


pub struct Virtq {
    inner: Arc<Mutex<VirtqInner<'_>>>,
}

impl Virtq {
    pub fn default() -> Virtq {
        Virtq {
            inner: Arc::new(Mutex::new(VirtqInner::default())),
        }
    }

    pub fn reset(&self, index: usize) {
        let mut inner = self.inner.lock();
        inner.reset(index);
    }

    pub fn pop_avail_desc_idx(&self, avail_idx: u16) -> Option<u16> {
        let mut inner = self.inner.lock();
        match &inner.avail {
            Some(avail) => {
                if avail_idx == inner.last_avail_idx {
                    return None;
                }
                let idx = inner.last_avail_idx as usize % inner.num;
                let avail_desc_idx = avail.ring[idx];
                inner.last_avail_idx = inner.last_avail_idx.wrapping_add(1);
                return Some(avail_desc_idx);
            }
            None => {
                println!("pop_avail_desc_idx: failed to avail table");
                return None;
            }
        }
    }


    pub fn put_back_avail_desc_idx(&self) {
        let mut inner = self.inner.lock();
        match &inner.avail {
            Some(_) => {
                inner.last_avail_idx -= 1;
            }
            None => {
                println!("put_back_avail_desc_idx: failed to avail table");
            }
        }
    }

    pub fn desc_is_writable(&self, idx: usize) -> bool {
        let inner = self.inner.lock();
        let desc_table = inner.desc_table.as_ref().unwrap();
        desc_table[idx].flags & VIRTQ_DESC_F_WRITE as u16 != 0
    }

    pub fn desc_has_next(&self, idx: usize) -> bool {
        let inner = self.inner.lock();
        let desc_table = inner.desc_table.as_ref().unwrap();
        desc_table[idx].flags & VIRTQ_DESC_F_NEXT != 0
    }
}

pub struct VirtqInner<'a> {
    ready: usize,
    vq_index: usize,
    num: usize,
    desc_table: Option<&'a mut [VringDesc]>,
    avail: Option<&'a mut VringAvail>,
    used: Option<&'a mut VringUsed>,
    last_avail_idx: u16,
    last_used_idx: u16,
    used_flags: u16,

    desc_table_addr: usize,
    avail_addr: usize,
    used_addr: usize,
}

impl VirtqInner<'_> {
    pub fn default() -> Self {
        VirtqInner {
            ready: 0,
            vq_index: 0,
            num: 0,
            desc_table: None,
            avail: None,
            used: None,
            last_avail_idx: 0,
            last_used_idx: 0,
            used_flags: 0,

            desc_table_addr: 0,
            avail_addr: 0,
            used_addr: 0,

            notify_handler: None,
        }
    }

    // virtio_queue_reset
    pub fn reset(&mut self, index: usize) {
        self.ready = 0;
        self.vq_index = index;
        self.num = 0;
        self.last_avail_idx = 0;
        self.last_used_idx = 0;
        self.used_flags = 0;
        self.desc_table_addr = 0;
        self.avail_addr = 0;
        self.used_addr = 0;

        self.desc_table = None;
        self.avail = None;
        self.used = None;
    }
}

