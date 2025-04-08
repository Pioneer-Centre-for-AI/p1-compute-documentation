# P1 DTU HPC

## Overview
The P1 DTU HPC is hosted at DTU and provides high-performance computing resources for P1 members. It is particularly suitable for medium to large-scale machine learning experiments and research projects.

## Getting Access
Before accessing the P1 DTU HPC, you must first [register to become a member of P1](https://www.aicentre.dk/affiliation) by filling out this [form](https://da.surveymonkey.com/r/P1Affiliation).

### Account Setup

Fill out the DTU account request form: <https://forms.office.com/e/DG5qCfs6Wm>. Only signup using an official university email address is accepted.

The form will be processed by Henning Christiansen, head of DTU's compute center and you will receive account details via email once created.

### Accessing the Cluster
The compute cluster is accessible at `login9.hpc.dtu.dk` via SSH. Note that:

- Home directories have limited storage (50GB)
- Additional storage is available at `/dtu/p1/`
- Interactive node is available for package installation and test runs
- Heavy jobs should be submitted as batch jobs

#### Access from Outside DTU Network
1. Download Cisco AnyConnect VPN client
2. Go to https://dtubasen.dtu.dk and sign in via Azure multi-factor auth using your full DTU username
3. Set up multi-factor authentication
4. Connect to vpn.dtu.dk using AnyConnect
5. SSH to `login9.hpc.dtu.dk` using your DTU credentials

For persistent access, you can set up SSH keys:

1. Generate an SSH key using ssh-keygen
2. Copy the key: `ssh-copy-id -i ~/.ssh/keyname username@login9.hpc.dtu.dk`
3. Connect: `ssh username@login9.hpc.dtu.dk`

## Support
- **Technical Support:** For issues like broken or missing packages, contact <support@hpc.dtu.dk>
- **Policy Support:** For policy issues, contact <compute-governance-p1@aicentre.dk>
- **General Questions:** Use the `#compute` or `#ask-compute` channels on P1 Slack
- **Compute Coordinator:** Contact <bstja@dtu.dk> for general or technical compute-related questions

For more technical information, refer to the [P1 compute cluster documentation](https://www.hpc.dtu.dk/?page_id=5028) at DTU DCC. 

## Hardware Specifications
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