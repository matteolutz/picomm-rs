use gstreamer::{
    self as gst,
    glib::{WeakRef, object::ObjectExt},
};

pub struct VolumeHandle {
    inner: WeakRef<gst::Element>,
}

impl VolumeHandle {
    pub fn new(volume: &gst::Element) -> Self {
        Self {
            inner: volume.downgrade(),
        }
    }

    pub fn get_volume(&self) -> Option<f64> {
        self.inner
            .upgrade()
            .map(|e: gst::Element| e.property::<f64>("volume"))
    }
}
