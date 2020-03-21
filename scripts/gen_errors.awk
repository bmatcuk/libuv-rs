# run like awk -f scripts/gen_errors.awk target/rls/debug/build/libuv-sys2-02275bbf285602f5/out/bindings.rs > src/error.inc.rs

/^pub const uv_errno_t_UV_/ {
  name = substr($3, 15, length($3) - 15);
  errors[name] = substr($6, 1, length($6) - 1)
}

END {
  indent = "    ";
  nerrors = asorti(errors);

  print "#[allow(non_camel_case_types)]";
  print "#[derive(Clone, Copy, Debug, Eq, PartialEq)]";
  print "pub enum Error {";
  for (i = 1; i <= nerrors; i++)
    print indent errors[i] ",";
  print "}\n";

  print "impl crate::FromInner<uv::uv_errno_t> for Error {";
  print indent "fn from_inner(code: uv::uv_errno_t) -> Error {";
  print indent indent "match code {";
  for (i = 1; i <= nerrors; i++)
    print indent indent indent "uv::uv_errno_t_UV_" errors[i] " => Error::" errors[i] ",";
  print indent indent indent "_ => Error::UNKNOWN,";
  print indent indent "}";
  print indent "}";
  print "}\n";

  print "impl Error {";
  print indent "fn code(&self) -> uv::uv_errno_t {";
  print indent indent "match self {";
  for (i = 1; i <= nerrors; i++)
    print indent indent indent "Error::" errors[i] " => uv::uv_errno_t_UV_" errors[i] ",";
  print indent indent "}";
  print indent "}";
  print "}"
}
