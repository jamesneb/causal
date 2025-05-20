// ebpf/probe.c
#include <linux/bpf.h>
#include <bpf/bpf_helpers.h>

struct syscall_event {
    u64 id;
    u64 timestamp;
};

struct {
    __uint(type, BPF_MAP_TYPE_ARRAY);
    __uint(max_entries, 512);
    __type(key, u32);
    __type(value, u64);
} SYSCALL_COUNTS SEC(".maps");

SEC("tracepoint/raw_syscalls/sys_enter")
int sys_enter(struct syscall_event *ctx) {
    u32 syscall_id = ctx->id;
    u64 *count;
    
    // Increment syscall counter
    count = bpf_map_lookup_elem(&SYSCALL_COUNTS, &syscall_id);
    if (count) {
        __sync_fetch_and_add(count, 1);
    }
    
    return 0;
}

char LICENSE[] SEC("license") = "GPL";
