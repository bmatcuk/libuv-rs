# run like awk -f scripts/gen_handle_types.awk target/rls/debug/build/libuv-sys2-02275bbf285602f5/out/bindings.rs > src/handle_types.inc.rs

/^pub const uv_handle_type_UV_/ && !/UNKNOWN_HANDLE|HANDLE_TYPE_MAX/ {
  name = substr($3, 19, length($3) - 19);
  types[name] = substr($6, 1, length($6) - 1)
}

END {
  indent = "    ";
  ntypes = asorti(types);

  print "#[allow(non_camel_case_types)]";
  print "#[derive(Clone, Copy, Debug)]";
  print "pub enum HandleType {";
  for (i = 1; i <= ntypes; i++)
    print indent types[i] ",";
  print indent "UNKNOWN,";
  print "}\n";

  print "impl From<uv::uv_handle_type> for HandleType {";
  print indent "fn from(t: uv::uv_handle_type) -> HandleType {";
  print indent indent "match t {";
  for (i = 1; i <= ntypes; i++)
    print indent indent indent "uv::uv_handle_type_UV_" types[i] " => HandleType::" types[i] ",";
  print indent indent indent "_ => HandleType::UNKNOWN,";
  print indent indent "}";
  print indent "}";
  print "}\n";

  print "impl Into<uv_handle_type> for &HandleType {";
  print indent "fn into(self) -> uv::uv_handle_type {";
  print indent indent "match self {";
  for (i = 1; i <= ntypes; i++)
    print indent indent indent "HandleType::" types[i] " => uv::uv_handle_type_UV_" types[i] ",";
  print indent indent indent "_ => uv::uv_handle_type_UV_UNKNOWN_HANDLE,";
  print indent indent "}";
  print indent "}";
  print "}"
}
