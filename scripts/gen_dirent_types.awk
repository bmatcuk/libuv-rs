# run like awk -f scripts/gen_dirent_types.awk target/rls/debug/build/libuv-sys2-02275bbf285602f5/out/bindings.rs > src/fs/dirent_types.inc.rs

/^pub const uv_dirent_type_t_UV_DIRENT_/ && !/UNKNOWN/ {
  name = substr($3, 28, length($3) - 28);
  types[name] = substr($6, 1, length($6) - 1)
}

END {
  indent = "    ";
  ntypes = asorti(types);

  print "#[allow(non_camel_case_types)]";
  print "#[derive(Clone, Copy, Debug)]";
  print "pub enum DirentType {";
  for (i = 1; i <= ntypes; i++)
    print indent types[i] ",";
  print indent "UNKNOWN,";
  print "}\n";

  print "impl crate::FromInner<uv::uv_dirent_type_t> for DirentType {";
  print indent "fn from_inner(t: uv::uv_dirent_type_t) -> DirentType {";
  print indent indent "match t {";
  for (i = 1; i <= ntypes; i++)
    print indent indent indent "uv::uv_dirent_type_t_UV_DIRENT_" types[i] " => DirentType::" types[i] ",";
  print indent indent indent "_ => DirentType::UNKNOWN,";
  print indent indent "}";
  print indent "}";
  print "}\n";

  print "impl crate::IntoInner<uv::uv_dirent_type_t> for &DirentType {";
  print indent "fn into_inner(self) -> uv::uv_dirent_type_t {";
  print indent indent "match self {";
  for (i = 1; i <= ntypes; i++)
    print indent indent indent "DirentType::" types[i] " => uv::uv_dirent_type_t_UV_DIRENT_" types[i] ",";
  print indent indent indent "_ => uv::uv_dirent_type_t_UV_DIRENT_UNKNOWN,";
  print indent indent "}";
  print indent "}";
  print "}"
}
