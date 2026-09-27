pub fn limit_arenas() {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    unsafe {
        libc::mallopt(libc::M_ARENA_MAX, 2);
    }
}

pub fn trim() {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    unsafe {
        libc::malloc_trim(0);
    }
}
