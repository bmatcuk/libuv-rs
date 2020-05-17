pub(crate) enum AddlHandleData<'a> {
    AsyncData(crate::AsyncDataFields<'a>),
    CheckData(crate::CheckDataFields<'a>),
    FsEventData(crate::FsEventDataFields<'a>),
    FsPollData(crate::FsPollDataFields<'a>),
    IdleData(crate::IdleDataFields<'a>),
    PollData(crate::PollDataFields<'a>),
    PrepareData(crate::PrepareDataFields<'a>),
    ProcessData(crate::ProcessDataFields<'a>),
    SignalData(crate::SignalDataFields<'a>),
    StreamData(crate::StreamDataFields<'a>),
    TimerData(crate::TimerDataFields<'a>),
}
