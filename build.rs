fn main() {
    cc::Build::new()
        .file("c/test-runner.c")
        .object("/usr/lib/libmgba.so.0.9.0")
        .include("c/include")
        .compile("test-runner");
}
