pub(crate) enum ReqData {
    NoData,
    ConnectData(crate::ConnectDataFields),
    FsData(crate::FsDataFields),
    GetAddrInfoData(crate::GetAddrInfoDataFields),
    GetNameInfoData(crate::GetNameInfoDataFields),
    ShutdownData(crate::ShutdownDataFields),
    UdpSendData(crate::UdpSendDataFields),
    WorkData(crate::WorkDataFields),
    WriteData(crate::WriteDataFields),
}
