// In a new file: ebpf.rs
use aya::programs::{TracePoint, ProgramError};
use aya::{include_bytes_aligned, Bpf};
use anyhow::Result;

pub struct EbpfCollector {
    bpf: Bpf,
    tracepoints: Vec<TracePoint>,
}

impl EbpfCollector {
    pub fn new() -> Result<Self> {
        // Load eBPF program (compiled separately)
        let bpf_bytes = include_bytes_aligned!("../../ebpf/target/bpfel-unknown-none/release/lambda-probe");
        let mut bpf = Bpf::load(bpf_bytes)?;
        
        // Load and attach tracepoints
        let tracepoint_sys_enter: TracePoint = bpf.program_mut("sys_enter")?.try_into()?;
        tracepoint_sys_enter.load()?;
        tracepoint_sys_enter.attach("raw_syscalls", "sys_enter")?;
        
        let tracepoints = vec![tracepoint_sys_enter];
        
        Ok(Self { bpf, tracepoints })
    }
    
    pub fn get_syscall_metrics(&self) -> Result<Value> {
        // Read from eBPF map
        let syscall_map = self.bpf.map_mut("SYSCALL_COUNTS")?;
        
        // Collect metrics from map
        let mut syscall_metrics = serde_json::Map::new();
        for i in 0..512 { // Max syscall number
            if let Ok(count) = syscall_map.lookup(&i.to_ne_bytes(), 8) {
                let count = u64::from_ne_bytes(count.try_into().unwrap());
                if count > 0 {
                    syscall_metrics.insert(format!("syscall_{}", i), json!(count));
                }
            }
        }
        
        Ok(json!(syscall_metrics))
    }
}
