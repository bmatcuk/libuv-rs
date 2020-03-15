# run like awk -f scripts/gen_fs_types.awk target/rls/debug/build/libuv-sys2-02275bbf285602f5/out/bindings.rs > src/fs/fs_types.inc.rs

/^pub const uv_fs_type_UV_FS_/ && !/UNKNOWN/ {
  name = substr($3, 18, length($3) - 18);
  types[name] = substr($6, 1, length($6) - 1)
}

END {
  indent = "    ";
  ntypes = asorti(types);

  print "#[allow(non_camel_case_types)]";
  print "#[derive(Clone, Copy, Debug)]";
  print "pub enum FsType {";
  for (i = 1; i <= ntypes; i++)
    print indent types[i] ",";
  print indent "UNKNOWN,";
  print "}\n";

  print "impl crate::FromInner<uv::uv_fs_type> for FsType {";
  print indent "fn from_inner(t: uv::uv_fs_type) -> FsType {";
  print indent indent "match t {";
  for (i = 1; i <= ntypes; i++)
    print indent indent indent "uv::uv_fs_type_UV_FS_" types[i] " => FsType::" types[i] ",";
  print indent indent indent "_ => FsType::UNKNOWN,";
  print indent indent "}";
  print indent "}";
  print "}\n";

  print "impl crate::IntoInner<uv::uv_fs_type> for &FsType {";
  print indent "fn into_inner(self) -> uv::uv_fs_type {";
  print indent indent "match self {";
  for (i = 1; i <= ntypes; i++)
    print indent indent indent "FsType::" types[i] " => uv::uv_fs_type_UV_FS_" types[i] ",";
  print indent indent indent "_ => uv::uv_fs_type_UV_FS_UNKNOWN,";
  print indent indent "}";
  print indent "}";
  print "}"
}
