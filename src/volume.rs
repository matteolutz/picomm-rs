use gstreamer::{self as gst, glib::WeakRef};

pub struct VolumeHandle {
    inner: WeakRef<gst::Element>,
}
