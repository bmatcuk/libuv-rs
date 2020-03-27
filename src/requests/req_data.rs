pub(crate) enum ReqData {
    ConnectData(crate::ConnectDataFields),
    FsData(crate::FsDataFields),
    GetAddrInfoData(crate::GetAddrInfoDataFields),
    GetNameInfoData(crate::GetNameInfoDataFields),
    RandomData(crate::RandomDataFields),
    ShutdownData(crate::ShutdownDataFields),
    UdpSendData(crate::UdpSendDataFields),
    WorkData(crate::WorkDataFields),
    WriteData(crate::WriteDataFields),
}
