bitflags! {
    pub struct FsModeFlags: i32 {
        const SetUID = 0x4000;
        const SetGID = 0x2000;
        const Sticky = 0x1000;
        const OwnerRead = 0x400;
        const OwnerWrite = 0x200;
        const OwnerExecute = 0x100;
        const GroupRead = 0x40;
        const GroupWrite = 0x20;
        const GroupExecute = 0x10;
        const OthersRead = 0x4;
        const OthersWrite = 0x2;
        const OthersExecute = 0x1;
    }
}
