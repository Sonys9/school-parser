use core::arch::asm;

static LEN: usize = 16;
static mut STACK: [u8; 1024 * LEN] = [0u8; 1024 * LEN];

#[allow(static_mut_refs)]
pub fn align(func: unsafe extern "C" fn() -> !) {
    unsafe { 
        asm!(
            "mov rdi, rsp",
            "mov rsp, {stack_top}",
            "and rsp, -16",
            "call {func}",
            "ud2",
            stack_top = in(reg) &STACK as *const _ as usize + 1024 * LEN,
            func = in(reg) func,
            options(noreturn)
        );
    }
}