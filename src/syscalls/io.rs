use core::arch::asm;

pub fn terminal(message: &str) {
    let ptr = message.as_ptr();
    unsafe {
        asm!(
            "syscall",
            in("rax") 1,
            in("rdi") 1,
            in("rsi") ptr,
            in("rdx") message.len(),
            out("rcx") _,
            out("r11") _,
            lateout("rax") _,
        );
    };
}
