import marimo

__generated_with = "0.25.1"
app = marimo.App(width="medium")


@app.cell
def _():
    import math
    import os
    import socket
    import time

    import marimo as mo
    import matplotlib.pyplot as plt
    import torch

    return math, mo, os, plt, socket, time, torch


@app.cell
def _(mo, os, socket, torch):
    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    if device.type == "cuda":
        free, total = torch.cuda.mem_get_info()
        major, minor = torch.cuda.get_device_capability()
        gpu = f"{torch.cuda.get_device_name()} (sm_{major}{minor})"
        memory = f"{(total - free) / 2**30:.1f} of {total / 2**30:.1f} GiB in use"
    else:
        gpu, memory = "none: running on CPU", "-"
    job = os.environ.get("LSB_JOBID", "-")
    mo.md(
        f"""
        # GPU on P1 DTU HPC

        | | |
        |---|---|
        | host | `{socket.gethostname()}` |
        | LSF job | `{job}` |
        | GPU | `{gpu}` |
        | GPU memory | {memory} |
        | torch | `{torch.__version__}`, CUDA `{torch.version.cuda}` |
        """
    )
    return (device,)


@app.cell
def _(mo):
    mo.md("""
    ## Matrix multiply: CPU against GPU

    Numbers move between runs: the node is shared with other jobs.
    """)
    return


@app.cell
def _(mo):
    size = mo.ui.slider(start=1024, stop=16384, step=1024, value=4096, label="matrix size", show_value=True)
    dtype = mo.ui.dropdown(["float32", "bfloat16", "float16"], value="bfloat16", label="dtype")
    mo.hstack([size, dtype], justify="start")
    return dtype, size


@app.cell
def _(device, dtype, mo, size, time, torch):
    def tflops(dev, n, kind, repeats=10):
        a = torch.randn(n, n, device=dev, dtype=kind)
        b = torch.randn(n, n, device=dev, dtype=kind)
        a @ b
        if dev.type == "cuda":
            torch.cuda.synchronize()
        start = time.perf_counter()
        for _ in range(repeats):
            a @ b
        if dev.type == "cuda":
            torch.cuda.synchronize()
        seconds = (time.perf_counter() - start) / repeats
        return seconds, 2 * n**3 / seconds / 1e12

    cpu_n = min(size.value, 2048)
    if device.type == "cuda":
        gpu_n, gpu_kind = size.value, dtype.value
    else:
        gpu_n, gpu_kind = cpu_n, "float32"
    gpu_s, gpu_tf = tflops(device, gpu_n, getattr(torch, gpu_kind))
    cpu_s, cpu_tf = tflops(torch.device("cpu"), cpu_n, torch.float32, repeats=2)
    mo.md("\n".join([
        f"`{gpu_n}×{gpu_n}` `{gpu_kind}` matmul",
        "",
        "| where | per matmul | TFLOP/s |",
        "|---|---|---|",
        f"| {device.type.upper()}, {gpu_kind}, n={gpu_n} | {gpu_s * 1000:.2f} ms | {gpu_tf:.1f} |",
        f"| CPU, float32, n={cpu_n} | {cpu_s * 1000:.2f} ms | {cpu_tf:.2f} |",
        "",
        f"**{gpu_tf / cpu_tf:.0f}×** the CPU's throughput.",
    ]))
    return


@app.cell
def _(mo):
    mo.md("""
    ## Train a classifier on the GPU

    A small MLP learns to separate three interleaved spirals. The data is made on
    the GPU and never leaves it.
    """)
    return


@app.cell
def _(mo):
    steps = mo.ui.slider(start=200, stop=5000, step=200, value=1500, label="steps", show_value=True)
    width = mo.ui.slider(start=64, stop=1024, step=64, value=256, label="hidden width", show_value=True)
    train = mo.ui.run_button(label="train")
    mo.hstack([steps, width, train], justify="start")
    return steps, train, width


@app.cell
def _(device, math, torch):
    def spirals(n, classes=3, noise=0.15):
        label = torch.randint(0, classes, (n,), device=device)
        radius = torch.rand(n, device=device)
        angle = radius * 4 * math.pi + label * (2 * math.pi / classes)
        angle = angle + noise * torch.randn(n, device=device)
        points = torch.stack([radius * torch.cos(angle), radius * torch.sin(angle)], dim=1)
        return points, label

    return (spirals,)


@app.cell
def _(device, mo, spirals, steps, time, torch, train, width):
    mo.stop(not train.value, mo.md("Press **train**."))

    torch.manual_seed(0)
    x, y = spirals(200_000)
    model = torch.nn.Sequential(
        torch.nn.Linear(2, width.value),
        torch.nn.SiLU(),
        torch.nn.Linear(width.value, width.value),
        torch.nn.SiLU(),
        torch.nn.Linear(width.value, width.value),
        torch.nn.SiLU(),
        torch.nn.Linear(width.value, 3),
    ).to(device)
    optimiser = torch.optim.AdamW(model.parameters(), lr=3e-3)
    schedule = torch.optim.lr_scheduler.OneCycleLR(optimiser, max_lr=3e-3, total_steps=steps.value)

    losses = []
    start = time.perf_counter()
    for _step in mo.status.progress_bar(range(steps.value), title="training"):
        batch = torch.randint(0, x.shape[0], (8192,), device=device)
        with torch.autocast(device.type, dtype=torch.bfloat16, enabled=device.type == "cuda"):
            loss = torch.nn.functional.cross_entropy(model(x[batch]), y[batch])
        optimiser.zero_grad(set_to_none=True)
        loss.backward()
        optimiser.step()
        schedule.step()
        losses.append(loss.detach())
    losses = torch.stack(losses).float().cpu()
    elapsed = time.perf_counter() - start

    with torch.no_grad():
        accuracy = (model(x).argmax(1) == y).float().mean().item()
    return accuracy, elapsed, losses, model


@app.cell
def _(accuracy, device, elapsed, losses, mo, model, plt, spirals, steps, torch):
    with torch.no_grad():
        axis = torch.linspace(-1.1, 1.1, 400, device=device)
        grid = torch.cartesian_prod(axis, axis)
        regions = model(grid).argmax(1).reshape(400, 400).T.cpu()
        sample, label = spirals(3000)
        sample, label = sample.cpu(), label.cpu()

    figure, (left, right) = plt.subplots(1, 2, figsize=(11, 4.5))
    left.plot(losses.numpy(), linewidth=0.8)
    left.set_yscale("log")
    left.set_xlabel("step")
    left.set_ylabel("loss")
    left.set_title("training loss")
    right.imshow(regions, origin="lower", extent=(-1.1, 1.1, -1.1, 1.1), cmap="Pastel1", alpha=0.9)
    right.scatter(sample[:, 0], sample[:, 1], c=label, cmap="Set1", s=2)
    right.set_title(f"decision regions, accuracy {accuracy:.1%}")
    right.set_aspect("equal")
    figure.tight_layout()

    mo.vstack([
        mo.md(f"{steps.value} steps in **{elapsed:.1f} s** on `{device}`, {steps.value / elapsed:.0f} steps/s"),
        figure,
    ])
    return


if __name__ == "__main__":
    app.run()
