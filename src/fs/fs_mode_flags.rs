bitflags! {
    pub struct FsModeFlags: i32 {
        const SET_UID = 0x4000;
        const SET_GID = 0x2000;
        const STICKY = 0x1000;
        const OWNER_READ = 0x400;
        const OWNER_WRITE = 0x200;
        const OWNER_EXECUTE = 0x100;
        const GROUP_READ = 0x40;
        const GROUP_WRITE = 0x20;
        const GROUP_EXECUTE = 0x10;
        const OTHERS_READ = 0x4;
        const OTHERS_WRITE = 0x2;
        const OTHERS_EXECUTE = 0x1;
    }
}

bitflags! {
    pub struct FsAccessFlags: i32 {
        const OK = 0;
        const READ = 4;
        const WRITE = 2;
        const EXECUTE = 1;
    }
}
