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

- Home directories have limited storage (30GB)
- Additional storage is available at `/dtu/p1/`
- Interactive node is available for package installation and test runs
- Heavy jobs should be submitted as batch jobs

#### Access from Outside DTU Network
1. Download Cisco AnyConnect VPN client (see OpenConnect for Linux)
2. Go to https://dtubasen.dtu.dk and sign in via Azure multi-factor auth using your full DTU username
3. Set up multi-factor authentication
4. Connect to vpn.dtu.dk using AnyConnect
5. SSH to `login9.hpc.dtu.dk` using your DTU credentials

For persistent access, you can set up SSH keys:

```
# 1. Generate key
ssh-keygen -t ed25519 -f ~/.ssh/keyname

# 2. Copy public key
ssh-copy-id -i ~/.ssh/keyname.pub username@login9.hpc.dtu.dk

# 3. Connect
ssh -i ~/.ssh/keyname username@login9.hpc.dtu.dk
```

## Support
- **Technical Support:** For issues like broken or missing packages, contact <support@hpc.dtu.dk>
- **Policy Support:** For policy issues, contact <compute-governance-p1@aicentre.dk>
- **General Questions:** Use the `#compute` or `#ask-compute` channels on P1 Slack
- **Compute Coordinator:** Contact <bstja@dtu.dk> for general or technical compute-related questions

For more technical information, refer to the [P1 compute cluster documentation](https://www.hpc.dtu.dk/?page_id=5028) at DTU DCC. 

## Fair Use Policy

The following rules are in place to ensure fair use of the P1 DTU HPC:

- Maximum wall time: 72 hours
- Maximum number of GPUs in a job: 2 (one node)
- Maximum running jobs: ~50% of total available GPUs
- 500 gb of storage (+30gb in home directory)

If you have a project that requires more storage resources than the above, please contact the governance group
<compute-governance-p1@aicentre.dk> to discuss your needs.

!!! warning
The P1 DTU HPC is only intended for non-GDPR data e.g. public datasets, open benchmarks, etc. To be more
specific, you can still work on private datasets because your home directory is not shared with other users, but
data is not encrypted in a way to be GDPR compliant. For GDPR-compliant data processing, please use the P1 NGC HPC.

## Investing Your Compute Budget

If you have compute funding or are applying for compute funding, you are very welcome to contact [P1's compute coordinator](https://www.aicentre.dk/computing) for a consultation.

Generally, P1 can help purchasing hardware for you if it is to be a part of P1's clusters. In this setup you would have priority access to the hardware that you purchase, but if it is unused other researchers at P1 can use it as well. Most HPC hardware has an expected utilization time of approximately 5 years.

An example of a compute budget expenditure - a researcher might have 400k DKK for non-GDPR computing. In that case P1 can purchase a 2xA100 node, with the researcher getting priority access. With the last 100k DKK, we recommend a diversification of the researchers compute portfolio.

Using some of the 100k for personal computers with powerful GPU's
and reserving a significant portion of funds for later. This could be for storage or cloud compute. Having the budget to spring for cloud compute if peak pressure on other computing resources makes training a model in time for a submission deadline infeasible can be necessary.

## Hardware Specifications

- 7 Lenovo ThinkSystem SR665 V3 servers
- Each node specifications:
    - 2 AMD EPYC 9354 32-Core Processors
    - 768GB RAM
    - 2 NVIDIA H100 PCIe GPUs (80GB each)
- Storage: 60TiB shared storage
- Operating System: Alma Linux
- Scheduling Environment: LSF
- Resource Allocation:
    - 7 nodes available for batch jobs (queue: `p1`)
    - 1 node reserved for interactive usage (queue: `p1i`)
