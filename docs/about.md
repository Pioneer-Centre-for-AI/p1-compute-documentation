# Introduction

The **Pioneer Centre for Artificial Intelligence** (P1) is a Danish cross-university research centre dedicated to advancing AI research through collaboration.

This documentation provides a concise guide for P1 affiliates on how to access and use our dedicated high-performance computing (HPC) facilities, as well as those of our partner institutions.

![Alt Text](figures/observatory.avif)

A central mission of P1 is to foster broad research collaboration, both within the centre and across its affiliated universities. Our HPC resources are designed to support P1 researchers at partner institutions, enabling projects of varying scale and complexity.

For more information, visit the [P1 Homepage](https://www.aicentre.dk/) or [P1 Computing](https://www.aicentre.dk/computing).

# Available Compute Resources

While we prioritize smaller projects as they represent the majority of research needs, we also support larger projects through our tiered compute resources, including the capability to train very large models and support European-level workloads on Tier 2/1 facilities. But, very large scale projects might be better supported within their own funding.

In practical terms this means that while training a large language or vision model from scratch might not be suitable for our Tier 3 facilities, such projects can be accommodated on a Tier 2 national facility (ex. Gefion HPC), which is specifically designed for large-scale model training and can even support some European-level (Tier 1) workloads.

## Support

In general the P1 does not have official support staff. Instead we rely on the community to help each other. However, for questions regarding the cluster, we have the following two contact e-mails:

* For technical issues, like broken or missing packages, contact <support@hpc.dtu.dk> 
* For policy issues, like priority access, queuing system etc. contact <compute-governance-p1@aicentre.dk>

If your submission to join the cluster has been pending for more than a week, you can contact the member of the P1 HPC 
committee of your institution. General announcements regarding the cluster will be made in the `#compute` channel on the 
official P1-slack. Similarly if you have general questions regarding compute, including using the P1-cluster, feel free 
to post those questions in the `#ask-compute` channel.

Institution | Name                                                                              | E-mail
----------- | --------------------------------------------------------------------------------- | ---
DTU         | [Nicki Skafte Detlefsen](https://orbit.dtu.dk/en/persons/nicki-skafte-detlefsen)  | <nsde@dtu.dk>
ITU         | [Rob van der Goot](https://pure.itu.dk/en/persons/rob-van-der-goot)               | <robv@itu.dk>
KU          | [Mikkel Fruelund Odgaard](https://di.ku.dk/ansatte/?pure=da/persons/770606)       | <miod@di.ku.dk>
AAU         | [Andreas Aakerberg](https://vbn.aau.dk/da/persons/132677)                         | <anaa@create.aau.dk>
AU          | [Akhil Arora](https://www.au.dk/en/akhil.arora@cs.au.dk)                          | <akhil.arora@cs.au.dk>

## Hierarchy of Compute Resources

Depending on what you are doing, you will need different levels of compute. The following table gives an overview of the different levels of compute and what they are suitable for.

Tier | Organization level | Description
---- | ------------------ | -----------
4    | Personal           | Personal laptop and/or desktop with either no GPUs or a single one. At this level compute is limited and mostly consists of code development and running experiments with small datasets and models.
3    | Institute          | The next level is compute clusters containing either several smaller GPUs or a few larger ones. Multiple experiments can be run in parallel and models can be parallelized over a few GPUs, but it is not feasible to train very large models from scratch. Expect limitations to wall time (a couple of days), storage (around ½-1 TB) and number of nodes/GPUs used (around 2-16).
2    | National           | National HPC facilities provide the necessary resources for large-scale model training. Expect fewer restrictions to wall time (1-2 weeks), storage (around 1-10 TB) and number of nodes/GPUs used (around 16-64). Some national facilities can also support European-level (Tier 1) workloads. For example, P1's Gefion HPC is a national facility that can support both Tier 2 and possibly Tier 1 workloads.
1    | European           | European HPC centers offer significantly larger scale compute resources, typically providing easy access to 8-16 GPUs with the possibility of scaling up to 500-800 GPUs depending on project requirements and the specific HPC center. Unlike lower tiers, walltime is usually not restricted; instead, limitations are set on total GPU hours used. See [Karolina](https://eurohpc-ju.europa.eu/supercomputers/our-supercomputers_en#karolina) and [Meluxina](https://eurohpc-ju.europa.eu/supercomputers/our-supercomputers_en#meluxina) for examples of European HPC centers with GPUs.
0    | Global             | The highest level of compute. Possibility to use above 1000 GPUs and perform calculations at exascale. Usually only available to large research projects and collaborations with industry. See [LUMI](https://eurohpc-ju.europa.eu/supercomputers/our-supercomputers_en#lumi) for an example of a global HPC center with GPUs.

The P1 DTU and NGC HPCs sit at level 3 in the hierarchy, while Gefion HPC primarily operates at level 2 but can support some level 1 workloads, making it capable of supporting both national and some European-level research projects.
