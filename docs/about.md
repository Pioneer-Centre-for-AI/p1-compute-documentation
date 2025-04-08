# P1 Available Compute Resources

The goal of the P1 HPCs is to enable research (collaborations) as widely as possible within P1, supporting projects of various scales. While we prioritize smaller projects as they represent the majority of research needs, we also support larger projects through our tiered compute resources.

In practical terms this means that while training a large language or vision model from scratch might not be suitable for our Tier 3 facilities, such projects can be accommodated on our Tier 2 national facility (P1 Gefion HPC), which is specifically designed for large-scale model training and can even support some European-level (Tier 1) workloads.

## Hierarchy of Compute Resources

Depending on what you are doing, you will need different levels of compute. The following table gives an overview of the different levels of compute and what they are suitable for.

Tier | Organization level | Description
---- | ------------------ | -----------
4    | Personal           | Personal laptop and/or desktop with either no GPUs or a single one. At this level compute is limited and mostly consist of code development and running experiment with small datasets and models.
3    | Institute          | The next level is compute clusters containing either several smaller GPUs or a few larger once. Multiple experiments can be run in parallel and models can be parallelized over a few GPUs, but it is not feasible to train very large models from scratch. Expect limitations to wall time (a couple of days), storage (arund ½-1 TB) and number of nodes/GPUs used (around 2-16).
2    | National           | National HPC facilities provide the necessary resources for large-scale model training. Expect less restrictions to wall time (1-2 weeks), storage (around 1-10 TB) and number of nodes/GPUs used (around 16-64). Some national facilities can also support European-level (Tier 1) workloads. For example, P1's Gefion HPC is a national facility that can support both Tier 2 and possibly Tier 1 workloads.
1    | European           | European HPC centers offer significantly larger scale compute resources, typically providing easy access to 8-16 GPUs with the possibility of scaling up to 500-800 GPUs depending on project requirements and the specific HPC center. Unlike lower tiers, walltime is usually not restricted; instead, limitations are set on total GPU hours used. See [Karolina](https://eurohpc-ju.europa.eu/supercomputers/our-supercomputers_en#karolina) and [Meluxina](https://eurohpc-ju.europa.eu/supercomputers/our-supercomputers_en#meluxina) for examples of European HPC centers with GPUs.
0    | Global             | The highest level of compute. Possibility to above 1000 GPU's and doing calculations at exoscale. Usually only available to large research projects and collaborations with industry. See [LUMI](https://eurohpc-ju.europa.eu/supercomputers/our-supercomputers_en#lumi) for an example of a global HPC center with GPUs.

The P1 DTU and NGC HPCs sit at level 3 in the hierarchy, while the P1 Gefion HPC primarily operates at level 2 but can support some level 1 workloads, making it capable of supporting both national and some European-level research projects.
