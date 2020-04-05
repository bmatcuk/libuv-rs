bitflags! {
    pub struct FsModeFlags: i32 {
        const SET_UID = 0o4000;
        const SET_GID = 0o2000;
        const STICKY = 0o1000;
        const OWNER_READ = 0o400;
        const OWNER_WRITE = 0o200;
        const OWNER_EXECUTE = 0o100;
        const GROUP_READ = 0o40;
        const GROUP_WRITE = 0o20;
        const GROUP_EXECUTE = 0o10;
        const OTHERS_READ = 0o4;
        const OTHERS_WRITE = 0o2;
        const OTHERS_EXECUTE = 0o1;
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
