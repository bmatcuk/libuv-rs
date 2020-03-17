pub(crate) enum ReqData {
    NoData,
    ConnectData(crate::ConnectDataFields),
    FsData(crate::FsDataFields),
    GetAddrInfoData(crate::GetAddrInfoDataFields),
    ShutdownData(crate::ShutdownDataFields),
    UdpSendData(crate::UdpSendDataFields),
    WorkData(crate::WorkDataFields),
    WriteData(crate::WriteDataFields),
}
