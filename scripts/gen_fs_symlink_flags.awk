# run like awk -f scripts/gen_fs_symlink_flags.awk target/rls/debug/build/libuv-sys2-02275bbf285602f5/out/bindings.rs > src/fs/fs_symlink_flags.inc.rs

/^pub const UV_FS_SYMLINK_/ {
  name = substr($3, 15, length($3) - 15);
  types[name] = substr($6, 1, length($6) - 1)
}

END {
  indent = "    ";
  ntypes = asorti(types);

  print "#[allow(non_camel_case_types)]";
  print "bitflags! {"
  print indent "pub struct FsSymlinkFlags: i32 {";
  for (i = 1; i <= ntypes; i++)
    print indent indent "const " types[i] " = uv::UV_FS_SYMLINK_" types[i] " as _;";
  print indent "}";
  print "}\n";
}
