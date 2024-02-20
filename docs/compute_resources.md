# P1-Owned Compute Resources

The goal of the P1 HPC is to enable research (collaborations) as widely as possible within P1. However, the goal is not 
to support very large projects, as this would hinder the usability for other users (we would rather support 10 smaller 
projects, then 1 large one). In practical terms this means that for example training a large language or vision model 
from scratch is not the intended use-case.

## Hierarchy of Compute Resources

Depending on what you are doing, you will need different levels of compute. The following table gives an overview of the
different levels of compute and what they are suitable for.

Tier | Organization level | Description
---- | ------------------ | -----------
4    | Personal           | Personal laptop and/or desktop with either no GPUs or a single one. At this level compute is limited and mostly consist of code development and running experiment with small datasets and models.
3    | Institute          | The next level is compute clusters containing either several smaller GPUs or a few larger once. Multiple experiments can be run in parallel and models can be parallelized over a few GPUs, but it is not feasible to train very large models from scratch. Expect limitations to wall time (a couple of days), storage (arund ½-1 TB) and number of nodes/GPUs used (around 2-16).
2    | National           | Denmark does currently not have a national HPC center with GPUs. At this level multiple experiments can be run in parallel and models can be parallelized over a few GPUs, but it is not feasible to train very large models from scratch. Expect less restrictions to wall time (1-2 weeks), storage (arund 1-10 TB) and number of nodes/GPUs used (around 16-64).
1    | European           | At this level it is feasible to train very large models from scratch. Getting access to this level of compute is usually done by applying for access to a European HPC center. Usually easy to get 8-16 GPU's, with possibliity to get up towards 500-800 GPU's depending on the project you are doing and the particular HPC center. Walltime is usually not restricted, instead limitations are set on total number of GPU hours used. See [Karolina](https://eurohpc-ju.europa.eu/supercomputers/our-supercomputers_en#karolina) and [Meluxina](https://eurohpc-ju.europa.eu/supercomputers/our-supercomputers_en#meluxina) for examples of European HPC centers with GPUs.
0    | Global             | The highest level of compute. Possibility to above 1000 GPU's and doing calculations at exoscale. Usually only available to large research projects and collaborations with industry. See [LUMI](https://eurohpc-ju.europa.eu/supercomputers/our-supercomputers_en#lumi) for an example of a global HPC center with GPUs.

The P1 cluster sits at level 3 in the hierarchy. The current hardware configuration consists of 7 Lenovo ThinkSystem 
SR665 V3 servers, each with 2 AMD EPYC 9354 32-Core Processors, and 768GB of RAM. Each node is equipped with 2 NVIDIA 
H100 PCIe GPU cards, with 80 GB of memory each. The Operating System is Alma Linux (ver. 9.2), and the servers are under 
control of a scheduling environment (LSF), to manage the concurrent workload of several simultaneous users. 

5 of the servers are available for batch jobs and in a queue named `p1`. 2 of the servers are reserved for interactive 
usage on a queue called `p1i`, and should be used for setting up the models, the computational environment, and for 
everything that needs interactive access.

The servers are connected to a 60TiB storage, accessible under /dtu/p1, this is reserved to P1 users.  

## Fair Use 

The following rules are in place to ensure fair use of the P1 HPC:

* maximum wall time: 72 hours
* maximum amount of nodes/GPU's used: 50% of the available resources
* 500 gb of storage (+50gb in home directory)

If you have a project that requires more resources than the above, please contact the governance group 
<compute-governance-p1@aicentre.dk> to discuss your needs.

## Investing Your Compute Budget

If you have compute funding or are applying for compute funding, you are very welcome to contact P1's compute 
coordinator for a consultation.

Generally, P1 can help purchasing hardware for you if it is to be a part of P1's clusters. In this setup you would have 
priority access to the hardware that you purchase, but if it is unused other researchers at P1 can use it as well. 
Most HPC hardware has an expected utilization time of approximately 5 years.

An example of a compute budget expenditure - a researcher might have 400k DKK for non-GDPR computing. In that case P1 
can purchase a 2xA100 node, with the researcher getting priority access. With the last 100k DKK, we recommend a 
diversification of the researchers compute portfolio. Using some of the 100k for personal computers with powerful GPU's 
and reserving a siginificant portion of funds for later. This could be for storage or cloud compute. Having the budget 
to spring for cloud compute if peak pressure on other computing resources makes training a model in time for a 
submission deadline infeasible can be necessary.
