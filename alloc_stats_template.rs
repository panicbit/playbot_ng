#![allow(unreachable_code)]
{crate_attrs}

fn main() {{
    __STAT_ALLOC.reset();

    {{
        {code}
    }};

    __STAT_ALLOC.print_stats();
}}

// CODE BELOW IS ALLOC STAT BOILERPLATE

#[global_allocator]
static __STAT_ALLOC: self::__stat_alloc::StatAlloc = self::__stat_alloc::StatAlloc {{
    allocator: std::alloc::System,
    num_allocs: std::sync::atomic::AtomicUsize::new(0),
    num_deallocs: std::sync::atomic::AtomicUsize::new(0),
    bytes_allocd: std::sync::atomic::AtomicUsize::new(0),
    bytes_deallocd: std::sync::atomic::AtomicUsize::new(0),
}};

mod __stat_alloc {{
    use std::alloc::{{GlobalAlloc, System, Layout}};
    use std::sync::atomic::{{AtomicUsize, Ordering::SeqCst}};

    pub struct StatAlloc {{
        pub allocator: System,
        pub num_allocs: AtomicUsize,
        pub num_deallocs: AtomicUsize,
        pub bytes_allocd: AtomicUsize,
        pub bytes_deallocd: AtomicUsize,
    }}
    
    impl StatAlloc {{
        pub fn print_stats(&self) {{
            let num_allocs = self.num_allocs.load(SeqCst);
            let num_deallocs = self.num_deallocs.load(SeqCst);
            let bytes_allocd = self.bytes_allocd.load(SeqCst);
            let bytes_deallocd = self.bytes_deallocd.load(SeqCst);
            println!("  #allocs: {{}},   allocated bytes: {{}}", num_allocs, bytes_allocd);
            println!("#deallocs: {{}}, deallocated bytes: {{}}", num_deallocs, bytes_deallocd);
            println!("lost bytes: {{}}", bytes_allocd as isize - bytes_deallocd as isize);
        }}
        
        pub fn reset(&self) {{
            self.num_allocs.store(0, SeqCst);
            self.num_deallocs.store(0, SeqCst);
            self.bytes_allocd.store(0, SeqCst);
            self.bytes_deallocd.store(0, SeqCst);
        }}
        
        fn count_alloc(&self, size: usize) {{
            self.num_allocs.fetch_add(1, SeqCst);
            self.bytes_allocd.fetch_add(size, SeqCst);
        }}

        fn count_dealloc(&self, size: usize) {{
            self.num_deallocs.fetch_add(1, SeqCst);
            self.bytes_deallocd.fetch_add(size, SeqCst);
        }}
    }}
    
    unsafe impl GlobalAlloc for StatAlloc {{
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {{
            self.count_alloc(layout.size());
            self.allocator.alloc(layout)
        }}
    
        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {{
            self.count_dealloc(layout.size());
            self.allocator.dealloc(ptr, layout)
        }}
    
        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {{
            self.count_alloc(layout.size());
            self.allocator.alloc_zeroed(layout)
        }}
    
        unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {{
            self.count_dealloc(layout.size());
            self.count_alloc(new_size);
            self.allocator.realloc(ptr, layout, new_size)
        }}
    }}
}}
