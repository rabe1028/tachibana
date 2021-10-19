use core::marker::PhantomData;

#[repr(C)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub enum FrameBuffer {
    Rgb(FrameBufferPayload<Rgb>),
    Bgr(FrameBufferPayload<Bgr>),
}

pub trait ColorSpace {}

#[repr(C)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self {r, g, b}
    }
}

impl From<Bgr> for Rgb {
    fn from(bgr: Bgr) -> Rgb {
        Rgb::new(bgr.r, bgr.g, bgr.b)
    }
}

impl ColorSpace for Rgb {}

#[repr(C)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct Bgr {
    pub b: u8,
    pub g: u8,
    pub r: u8,
}

impl Bgr {
    pub fn new(b: u8, g: u8, r: u8) -> Self {
        Self {b, g, r}
    }
}

impl From<Rgb> for Bgr {
    fn from(rgb: Rgb) -> Bgr {
        Bgr::new(rgb.b, rgb.g, rgb.r)
    }
}

impl ColorSpace for Bgr {}

#[repr(C)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct FrameBufferPayload<Color: ColorSpace> {
    pub frame_buffer: *mut u8,
    pub stride: u32,
    pub resolution: (u32, u32), // (horizontal, vertical)
    pub format: PhantomData<Color>,
}

impl<A: ColorSpace> FrameBufferPayload<A> {
    pub fn new(frame_buffer: *mut u8, stride: u32, resolution: (u32, u32)) -> Self {
        Self {
            frame_buffer, stride, resolution, format: PhantomData
        }
    }
}