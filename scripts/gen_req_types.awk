# run like awk -f scripts/gen_req_types.awk target/rls/debug/build/libuv-sys2-02275bbf285602f5/out/bindings.rs > src/req_types.inc.rs

/^pub const uv_req_type_UV_/ && !/UNKNOWN_REQ|REQ_TYPE_MAX/ {
  name = substr($3, 16, length($3) - 16);
  types[name] = substr($6, 1, length($6) - 1)
}

END {
  indent = "    ";
  ntypes = asorti(types);

  print "#[allow(non_camel_case_types)]";
  print "#[derive(Clone, Copy, Debug)]";
  print "pub enum ReqType {";
  for (i = 1; i <= ntypes; i++)
    print indent types[i] ",";
  print indent "UNKNOWN,";
  print "}\n";

  print "impl crate::FromInner<uv::uv_req_type> for ReqType {";
  print indent "fn from_inner(t: uv::uv_req_type) -> ReqType {";
  print indent indent "match t {";
  for (i = 1; i <= ntypes; i++)
    print indent indent indent "uv::uv_req_type_UV_" types[i] " => ReqType::" types[i] ",";
  print indent indent indent "_ => ReqType::UNKNOWN,";
  print indent indent "}";
  print indent "}";
  print "}\n";

  print "impl crate::IntoInner<uv::uv_req_type> for &ReqType {";
  print indent "fn into_inner(self) -> uv::uv_req_type {";
  print indent indent "match self {";
  for (i = 1; i <= ntypes; i++)
    print indent indent indent "ReqType::" types[i] " => uv::uv_req_type_UV_" types[i] ",";
  print indent indent indent "_ => uv::uv_req_type_UV_UNKNOWN_REQ,";
  print indent indent "}";
  print indent "}";
  print "}"
}
