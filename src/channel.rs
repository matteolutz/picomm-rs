#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub enum Channel {
    #[default]
    ChannelBroadcast,

    Channel1,
    Channel2,
    Channel3,
    Channel4,
}

impl Channel {
    pub fn get_multicast(self) -> (&'static str, u16) {
        match self {
            Channel::ChannelBroadcast => ("239.255.0.0", 5000),
            Channel::Channel1 => ("239.255.0.1", 5000),
            Channel::Channel2 => ("239.255.0.2", 5000),
            Channel::Channel3 => ("239.255.0.3", 5000),
            Channel::Channel4 => ("239.255.0.4", 5000),
        }
    }
}
