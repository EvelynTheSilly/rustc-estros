// FIXME(#142296): this hack is because there is no reliable way (yet) to determine whether a given
// target supports std. In the long-term, we should try to implement a way to *reliably* determine
// target (std) metadata.
//
// NOTE: this is pulled out to `build_helpers` to share this hack between `bootstrap` and
// `compiletest`.
pub fn target_supports_std(target_tuple: &str) -> bool {
    !(target_tuple.contains("-none")
        || target_tuple.contains("nvptx")
        || target_tuple.contains("switch"))
}

// Freestanding userlands: full std (an OS layer exists), but no libc, so the
// plain C names `memcpy`/`memmove`/`memset`/... that rustc's codegen
// unconditionally emits must come from `compiler_builtins`'s `mem` feature
// instead. Same classification limitation as #142296; keyed on the OS name in
// the triple.
pub fn target_needs_builtins_mem(target_tuple: &str) -> bool {
    target_tuple.ends_with("estros")
}
