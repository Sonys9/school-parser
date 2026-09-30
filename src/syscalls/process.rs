use core::arch::asm;

pub fn exit(code: i32) -> ! {
    unsafe {
        asm!(
            "syscall",
            in("rax") 60,
            in("rdi") code,
            options(noreturn)
        );
    };
}