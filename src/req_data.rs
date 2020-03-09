pub(crate) enum ReqData {
    NoData,
    ConnectData(crate::ConnectDataFields),
    ShutdownData(crate::ShutdownDataFields),
    UdpSendData(crate::UdpSendDataFields),
    WriteData(crate::WriteDataFields),
}
