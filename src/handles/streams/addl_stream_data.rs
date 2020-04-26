pub(crate) enum AddlStreamData<'a> {
    NoAddlStreamData,
    UdpData(crate::UdpDataFields<'a>),
}
