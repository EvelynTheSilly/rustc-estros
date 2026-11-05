use super::common::{abort_internal, cleanup};
use core::arch::naked_asm;

#[unsafe(no_mangle)]
#[unsafe(naked)]
extern "C" fn _start() {
    naked_asm!(
        "
        ldr x0, =_stack
        mov sp, x0
        bl {}
        svc #2
        b . // fallback
        ",
        sym _rust_main
    );
}
fn _rust_main() {
    unsafe extern "C" {
        fn main() -> i32;
    }

    unsafe {
        main();

        cleanup();
        abort_internal()
    }
}
