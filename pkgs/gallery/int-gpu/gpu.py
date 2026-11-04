"""Run this inside the job, from a GPU shell or a terminal inside VS Code:
`uv run gpu.py`. It shows the GPU the job holds and times one matrix multiply
on it against the CPU. It uses the example's own environment, with PyTorch."""
import os
import socket
import time

import torch

job = os.environ.get("LSB_JOBID", "-")
print(f"host {socket.gethostname()}, job {job}, torch {torch.__version__}")
if not torch.cuda.is_available():
    raise SystemExit("no GPU visible: run this inside the job, not on the login node")

free, total = torch.cuda.mem_get_info()
print(f"GPU {torch.cuda.get_device_name()}, {free / 2**30:.0f} of {total / 2**30:.0f} GiB free")


def matmul_seconds(device, n=4096, repeats=5):
    a = torch.randn(n, n, device=device, dtype=torch.float32)
    b = torch.randn(n, n, device=device, dtype=torch.float32)
    a @ b
    if device == "cuda":
        torch.cuda.synchronize()
    start = time.perf_counter()
    for _ in range(repeats):
        a @ b
    if device == "cuda":
        torch.cuda.synchronize()
    return (time.perf_counter() - start) / repeats


gpu, cpu = matmul_seconds("cuda"), matmul_seconds("cpu", repeats=1)
print(f"4096x4096 float32 matmul: GPU {gpu * 1000:.1f} ms, CPU {cpu * 1000:.0f} ms, {cpu / gpu:.0f}x faster")
