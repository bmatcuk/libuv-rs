pub(crate) enum ReqData<'a> {
    ConnectData(crate::ConnectDataFields<'a>),
    FsData(crate::FsDataFields<'a>),
    GetAddrInfoData(crate::GetAddrInfoDataFields<'a>),
    GetNameInfoData(crate::GetNameInfoDataFields<'a>),
    RandomData(crate::RandomDataFields<'a>),
    ShutdownData(crate::ShutdownDataFields<'a>),
    UdpSendData(crate::UdpSendDataFields<'a>),
    WorkData(crate::WorkDataFields<'a>),
    WriteData(crate::WriteDataFields<'a>),
}
