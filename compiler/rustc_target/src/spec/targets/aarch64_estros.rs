// the estros target
use rustc_abi::Endian;

use crate::spec::{
    Arch, Cc, CfgAbi, LinkerFlavor, Lld, Os, PanicStrategy, RelocModel, RustcAbi, SanitizerSet,
    StackProbeType, Target, TargetMetadata, TargetOptions,
};

// Default linker script for the estros userland: loads at 0x40000000 and
// places a 64K stack immediately after .bss (symbols `_start` and `_stack`).
const LINKER_SCRIPT: &str = include_str!("./aarch64_estros_linker_script.ld");

pub(crate) fn target() -> Target {
    // `-u _start` forces lld to pull `_start` out of the std rlib even though
    // nothing references it (it is the entry point).
    let pre_link_args =
        TargetOptions::link_args(LinkerFlavor::Gnu(Cc::No, Lld::No), &["-u", "_start"]);
    let opts = TargetOptions {
        os: Os::Estros,
        cfg_abi: CfgAbi::SoftFloat,
        rustc_abi: Some(RustcAbi::Softfloat),
        linker_flavor: LinkerFlavor::Gnu(Cc::No, Lld::Yes),
        linker: Some("rust-lld".into()),
        link_script: Some(LINKER_SCRIPT.into()),
        pre_link_args,
        features: "+v8a,+strict-align,-neon".into(),
        relocation_model: RelocModel::Static,
        disable_redzone: true,
        max_atomic_width: Some(128),
        supported_sanitizers: SanitizerSet::KCFI
            | SanitizerSet::KERNELADDRESS
            | SanitizerSet::KERNELHWADDRESS,
        stack_probes: StackProbeType::Inline,
        panic_strategy: PanicStrategy::Abort,
        endian: Endian::Little,
        ..Default::default()
    };
    Target {
        llvm_target: "aarch64-unknown-none".into(),
        metadata: TargetMetadata {
            description: Some("Bare ARM64, softfloat, for the estros OS".into()),
            tier: Some(2),
            host_tools: Some(false),
            std: Some(true),
        },
        pointer_width: 64,
        data_layout: "e-m:e-p270:32:32-p271:32:32-p272:64:64-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128-Fn32".into(),
        arch: Arch::AArch64,
        options: opts,
    }
}
