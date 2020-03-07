pub(crate) enum ReqData {
    NoData,
    ConnectData(crate::ConnectDataFields),
    ShutdownData(crate::ShutdownDataFields),
    WriteData(crate::WriteDataFields),
}
