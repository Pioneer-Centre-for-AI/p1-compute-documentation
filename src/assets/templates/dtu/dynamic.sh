#!/bin/bash

set -euo pipefail  # safety flags

# Read scheduler configuration from environment variables (dynamic with sensible defaults)
: "${SCHEDULER_JOB_NAME:?Must provide SCHEDULER_JOB_NAME}"
: "${SCHEDULER_EMAIL:?Must provide SCHEDULER_EMAIL}"
: "${SCHEDULER_CORES:=4}"
: "${SCHEDULER_WALLTIME:=01:00}"
: "${SCHEDULER_QUEUE:=p1}"
: "${SCHEDULER_MEMORY:=4GB}"
: "${SCHEDULER_GPUS:=1}"

# Executed on node: Embedded job script using heredoc
JOB_SCRIPT=$(cat <<'EOF'
#!/bin/bash
set -euxo pipefail

# Load the cuda module
module load cuda/11.6

# Check the GPU status
nvidia-smi

# Run the deviceQuery program
/appl/cuda/11.6.0/samples/bin/x86_64/linux/release/deviceQuery
EOF
)

# Executed on host: Submit the job using CLI arguments
bsub \
  -J "${SCHEDULER_JOB_NAME}" \
  -q "${SCHEDULER_QUEUE}" \
  -o "%J.out" \
  -e "%J.err" \
  -n "${SCHEDULER_CORES}" \
  -gpu "num=${SCHEDULER_GPUS}:mode=exclusive_process" \
  -R "rusage[mem=${SCHEDULER_MEMORY}]" \
  -R "span[hosts=1]" \
  -W "${SCHEDULER_WALLTIME}" \
  <<< "$JOB_SCRIPT"
