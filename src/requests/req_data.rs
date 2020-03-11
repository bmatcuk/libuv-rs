pub(crate) enum ReqData {
    NoData,
    ConnectData(crate::ConnectDataFields),
    FsData(crate::FsDataFields),
    ShutdownData(crate::ShutdownDataFields),
    UdpSendData(crate::UdpSendDataFields),
    WriteData(crate::WriteDataFields),
}
