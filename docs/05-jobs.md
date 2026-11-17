# Jobs

`aicentre list jobs` shows the user's jobs and `aicentre list gpus` shows how
busy the GPUs are. aicentre starts and ends
its own sessions ([04](04-sessions.md)) and lists every job, but it does not
submit, follow or cancel batch jobs.

## No batch commands since 0.2

0.1 had `submit`, `logs` and `cancel`, and the MCP tools `p1_submit`,
`p1_logs` and `p1_cancel`. 0.2 dropped them to keep aicentre to sessions,
listing, data and the docs. Each command now exits 2 and names the
scheduler's own command, to run in `aicentre shell`: `bsub`, `bjobs` and
`bkill` on DTU, `sbatch`, `squeue` and `scancel` on Gefion. A GPU session
still ends its job on Ctrl-C or at the idle limit.

0.1 kept a record per submitted job in `$XDG_STATE_HOME/aicentre/jobs/`. 0.2
reads none, and `uninstall` removes the state directory with them in it.

## list jobs

`aicentre list jobs` shows the user's unfinished jobs across P1 compute in one
table, with the provider as one column (`src/jobs/`). Every provider in use is
asked at the same time.

- DTU: `bjobs -noheader -o 'jobid stat queue job_name exec_host run_time time_left pend_reason delimiter='|''`.
- Gefion: `squeue --noheader -u "$USER" --format='%i|%T|%P|%j|%N|%M|%L|%r'`.
  `-u` rather than `--me`, which older Slurm lacks. `$USER` expands on the
  login node.

The free-text reason goes last in both, because it may hold the delimiter.
bjobs reports "no unfinished job" on stderr, with exit 0 or 255 depending on
the version, so that text counts as an empty list. Both are read into one
`Job` shape (`src/jobs/types.rs`): state running, pending, suspended or other,
the scheduler's own state word, queue, name, first node, time used and time
left as `h:mm:ss` or `d-hh:mm:ss`, and the pending reason. Running jobs come
first.

The parsers handle the schedulers' own formats. bjobs prints `exec_host` as
`4*n-62-30-1:2*n-62-30-2`, `run_time` as `300 second(s)` and `time_left` as
`0:55 L` (hours and minutes, then a letter for the kind of limit). DTU's LSF
is set up to print both as `00:04:40`, with seconds, and the parser takes
either. squeue
prints nodes as `dgx-[01-02]` or `dgx-01,dgx-02`, durations as `1-02:03:04`,
`2:03:04` or `03:04`, and `0:00`, `UNLIMITED` or `INVALID` for no value. The
first node and `h:mm:ss` come out of all of them.

Providers are asked at once, so nothing may prompt. The query goes through
the shared login when it is open, and otherwise runs with `BatchMode=yes` and
`ConnectTimeout=10`. An expired Teleport certificate is reported straight
away, without the 30 seconds ssh takes to find out. When a provider wants a
password or a new host key trusted, and a person is at the terminal, aicentre
logs in to each such provider on the terminal, one at a time, then asks it
again through that login.

NGC is air-gapped, so only its remote desktop can see its queue. When NGC is
in use, the list says so, so its jobs are not silently missing. Jobs this
laptop has a GPU session attached to are marked. `--json` prints the jobs and
each provider's status.

## list gpus

`aicentre list gpus` answers "is there room?" before a session or a batch job
is started (`src/jobs/gpus.rs`). It reuses the login, the provider choice and
the error handling of `list jobs`, and `--json` prints `gpus`, `queues` and
`providers`.

On DTU one login-node command asks both queues, `p1i` for sessions and `p1`
for batch jobs:

```sh
for q in p1i p1; do
  echo "@@QUEUE $q"
  bqueues -noheader -o "queue_name status pend run delimiter='|'" "$q"
  bhosts -gpu $(bqueues -l "$q" | sed -n 's/^HOSTS: *//p' | tr -d /)
done
```

The queue's host group comes from `bqueues -l`, so a node added to the queue
shows up without a release. `bhosts -gpu` prints one line per GPU with the
memory in use and the number of jobs on it; the host name is only on a
node's first line. Each queue gets a heading with its counts, then a table
with one row per GPU, numbers right-aligned and memory as `35.3 GB`. On `p1i` the GPUs are shared (`-gpu mode=shared`), so several
sessions sit on one GPU and a new one starts at once; what a user needs to
see there is the memory already taken, and a line under the table says so.
On `p1` a GPU belongs to one job, so the heading counts GPUs with no job as
free, next to the jobs running and waiting, and such a GPU's row says
`free`.

On Gefion it is `sinfo -h -o '%P|%D|%T|%G'`, and the line per partition
counts idle and partly used nodes and names the GPUs per node. This half has
only run against the test fake, not against Gefion.
