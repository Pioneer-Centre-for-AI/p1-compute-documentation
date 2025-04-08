# Hardware Specifications

## P1 DTU HPC

- 7 Lenovo ThinkSystem SR665 V3 servers
- Each node specifications:
  - 2 AMD EPYC 9354 32-Core Processors
  - 768GB RAM
  - 2 NVIDIA H100 PCIe GPUs (80GB each)
- Storage: 60TiB shared storage
- Operating System: Alma Linux 9.2
- Scheduling Environment: LSF
- Resource Allocation:
  - 5 servers available for batch jobs (queue: `p1`)
  - 2 servers reserved for interactive usage (queue: `p1i`)

## P1 NGC HPC

- Air-gapped system for secure data processing and is GDPR compliant
- Secure storage solutions
- Specific hardware details available upon access approval
- Scheduling Environment: SLURM
- Resource allocation details provided during onboarding

## P1 Gefion HPC

- NVIDIA DGX SuperPOD configuration
- Multiple DGX nodes with high-performance interconnects
- Large-scale GPU resources for distributed training
- High-bandwidth storage solutions
- Scheduling Environment: SLURM
