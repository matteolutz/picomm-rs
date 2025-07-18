use gstreamer::{
    self as gst,
    glib::{WeakRef, object::ObjectExt},
};

#[derive(Debug)]
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

    pub fn set_volume(&mut self, volume: f64) {
        if let Some(element) = self.inner.upgrade() {
            element.set_property("volume", &volume);
        }
    }

    pub fn mute(&self) {
        if let Some(element) = self.inner.upgrade() {
            element.set_property("mute", &true);
        }
    }

    pub fn unmute(&self) {
        if let Some(element) = self.inner.upgrade() {
            element.set_property("mute", &false);
        }
    }
}
