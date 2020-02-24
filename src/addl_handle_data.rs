pub(crate) enum AddlHandleData {
    NoAddlData,
    TimerData(crate::TimerDataFields),
    PrepareData(crate::PrepareDataFields),
    CheckData(crate::CheckDataFields),
    IdleData(crate::IdleDataFields),
    AsyncData(crate::AsyncDataFields),
    SignalData(crate::SignalDataFields),
}
