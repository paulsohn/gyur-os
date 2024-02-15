const KB: usize = 0x400;
const MB: usize = KB * KB;
const GB: usize = MB * KB;

const KERNEL_PAGE_SIZE: usize = 4 * KB;

#[derive(Clone, Copy)]
#[repr(C, align(0x1000))]
pub struct Page([u8; KERNEL_PAGE_SIZE]);
impl Page {
    pub const fn new() -> Self {
        Self([0; KERNEL_PAGE_SIZE])
    }
}