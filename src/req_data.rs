pub(crate) enum ReqData {
    NoData,
    ShutdownData(crate::ShutdownDataFields),
    WriteData(crate::WriteDataFields),
}
