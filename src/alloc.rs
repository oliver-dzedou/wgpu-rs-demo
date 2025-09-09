use mimalloc::MiMalloc;

/// Replaces the global allocator with [mimalloc]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
