pub(crate) enum AddlHandleData {
    NoAddlData,
    AsyncData(crate::AsyncDataFields),
    CheckData(crate::CheckDataFields),
    FsEventData(crate::FsEventDataFields),
    FsPollData(crate::FsPollDataFields),
    IdleData(crate::IdleDataFields),
    PrepareData(crate::PrepareDataFields),
    ProcessData(crate::ProcessDataFields),
    SignalData(crate::SignalDataFields),
    StreamData(crate::StreamDataFields),
    TimerData(crate::TimerDataFields),
}
