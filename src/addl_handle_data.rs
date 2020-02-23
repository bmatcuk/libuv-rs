pub(crate) enum AddlHandleData {
    NoAddlData,
    TimerData(crate::TimerDataFields),
    PrepareData(crate::PrepareDataFields),
    CheckData(crate::CheckDataFields),
    IdleData(crate::IdleDataFields),
}
