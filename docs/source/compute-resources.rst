P1-Owned Compute Resources
##########################

The current hardware configuration consists of 7 Lenovo ThinkSystem SR665 V3 servers, each with 2 AMD EPYC 9354 32-Core Processors, and 768GB of RAM. Each node is equipped with 2 NVIDIA H100 PCIe GPU cards, with 80 GB of memory each.

The Operating System is Alma Linux (ver. 9.2), and the servers are under control of a scheduling environment (LSF), to manage the concurrent workload of several simultaneous users. 

5 of the servers are available for batch jobs and in a queue named p1. 
2 of the servers are reserved for interactive usage, and should be used for setting up the models, the computational environment, and for everything that needs interactive access.

The servers are connected to a 60TiB storage, accessible under /dtu/p1, this is reserved to P1 users.  

Fair Use 
########
The goal of the P1 HPC is to enable research (collaborations) as widely as
possible within P1. However, the goal is not to support very large projects, as
this would hinder the usability for other users (we would rather support 10
smaller projects, then 1 large one). In practical terms this means that for
example training a large language or vision model from scratch is not the
intended use-case.

With these goals in mind, we have set the following limitations: 

* maximum wall time: 72 hours
* maximum amount of nodes/GPU's used: 50% of the available resources
* 500 gb of storage (+50gb in home directory)

Investing Your Compute Budget
*****************************
If you have compute funding or are applying for compute funding, you are very welcome to contact P1's compute coordinator for a consultation.

Generally, P1 can help purchasing hardware for you if it is to be a part of P1's clusters. 
In this setup you would have priority access to the hardware that you purchase, but if it is unused other researchers at P1 can use it as well. 
Most HPC hardware has an expected utilization time of approximately 5 years.

An example of a compute budget expenditure - a researcher might have 400k DKK for non-GDPR computing. 
In that case P1 can purchase a 2xA100 node, with the researcher getting priority access. 
With the last 100k DKK, we recommend a diversification of the researchers compute portfolio. 
Using some of the 100k for personal computers with powerful GPU's and reserving a siginificant portion of funds for later. 
This could be for storage or cloud compute. 
Having the budget to spring for cloud compute if peak pressure on other computing resources 
makes training a model in time for a submission deadline infeasible can be necessary.

