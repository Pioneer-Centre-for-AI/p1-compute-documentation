#!/bin/bash
#BSUB -J my_job_name
#BSUB -q p1
#BSUB -o %J.out
#BSUB -e %J.err
#BSUB -n 8
#BSUB -gpu "num=2:mode=exclusive_process"
#BSUB -R "rusage[mem=4GB]"
#BSUB -R "span[hosts=1]"
#BSUB -W 01:00

# Load the cuda module
module load cuda/11.6

# Check the GPU status
nvidia-smi

# Run the deviceQuery program
/appl/cuda/11.6.0/samples/bin/x86_64/linux/release/deviceQuery
