# Regenerating Files
Build the code with `cargo build`, then find the latest libuv-sys bindings
with:

```bash
# results sorted newest to oldest
find target/ -name bindings.rs -type f -exec ls -1lt "{}" +;
```

Each `get_*.awk` file then contains instructions at the top - substitute the
bindings.rs path found above in the example at the top of each `gen_*.awk`
file.
