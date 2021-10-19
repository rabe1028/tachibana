#![no_std]
#![no_main]
#![feature(asm)]
#![feature(lang_items)]

use tachibana_common::frame_buffer::{Bgr, ColorSpace, FrameBuffer, FrameBufferPayload, Rgb};

#[no_mangle]
pub extern "sysv64" fn kernel_main(fb: &mut FrameBuffer) {
    match fb {
        FrameBuffer::Rgb(payload) => render_example(payload),
        FrameBuffer::Bgr(payload) => render_example(payload),
    }

    loop {
        unsafe { asm!("hlt") }
    }
}

trait PixelWrite<A: ColorSpace> {
    fn put_pixel(&self, x: u32, y: u32, color: impl Into<A>);
}

impl PixelWrite<Rgb> for FrameBufferPayload<Rgb> {
    fn put_pixel(&self, x: u32, y: u32, color: impl Into<Rgb>) {
        let color = color.into();
        unsafe {
            let offset = (4 * (self.stride * y + x)) as usize;
            *self.frame_buffer.add(offset) = color.r;
            *self.frame_buffer.add(offset + 1) = color.g;
            *self.frame_buffer.add(offset + 2) = color.b;
        }
    }
}

impl PixelWrite<Bgr> for FrameBufferPayload<Bgr> {
    fn put_pixel(&self, x: u32, y: u32, color: impl Into<Bgr>) {
        let color = color.into();
        unsafe {
            let offset = (4 * (self.stride * y + x)) as usize;
            *self.frame_buffer.add(offset) = color.r;
            *self.frame_buffer.add(offset + 1) = color.g;
            *self.frame_buffer.add(offset + 2) = color.b;
        }
    }
}

fn render_example<A: ColorSpace>(fb: &mut FrameBufferPayload<A>)
where
    A: ColorSpace,
    Rgb: Into<A>,
    FrameBufferPayload<A> : PixelWrite<A>
{
    let color = Rgb::new(255, 255, 255);
    for x in 0..fb.resolution.0 {
        for y in 0..fb.resolution.1 {
            fb.put_pixel(x, y, color.clone());
        }
    }

    let color = Rgb::new(50, 155, 255);

    for x in 50..250 {
        for y in 50..150 {
            fb.put_pixel(x, y, color.clone());
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        unsafe { asm!("hlt") }
    }
}
