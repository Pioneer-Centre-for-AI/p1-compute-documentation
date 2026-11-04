# Batch and jobs

## submit

`aicentre submit <script> [--dir path]` sends a job script to the batch queue
of the provider it was written for (`src/batch/submit.ts`). Its directives
name the scheduler, which names the provider (`src/batch/script.ts`):

| Directive | Scheduler | Provider |
| --- | --- | --- |
| `#BSUB` | LSF | DTU |
| `#SBATCH` | Slurm | Gefion |
| `#PBS` | PBS | NGC, refused: it is air-gapped, so the user runs `qsub` on its remote desktop |

So `submit train.sh` needs no `-p`. When the script has directives, `-p` only
has to agree with them. A plain script goes to the provider `-p` names, or the
first in use.

A script on this machine is copied to `<data>/jobs/<timestamp>-<name>` first.
A path that is not here is taken to be on the cluster already, and is read
there for its directives. The job runs in `--dir`, which defaults to the
provider's project directory.

P1's defaults fill in what the script leaves out:

- DTU: `-q p1`, since without it a job lands in DTU's general queue. `-o` and
  `-e` go to `<data>/logs/%J.out` and `.err`, where `logs` looks. bsub reads
  `#BSUB` lines only from a script on stdin, so the command is
  `cd <dir> && bsub <flags> < <script>`.
- Gefion: `sbatch --parsable --chdir=<dir>`, `--account=iu_0096` (P1's Slurm
  account) and `--output=<data>/logs/%j.out`. `account` in `config.toml`
  replaces the default, and a script's own `#SBATCH --account` or `-A` wins
  over both.

A directive counts as set in its short or long form, and as `--opt=value`.

Each submitted job leaves a record in
`$XDG_STATE_HOME/aicentre/jobs/<provider>-<id>.json` with its name, output
and error files, and time. `logs 123` and `cancel 123` then know where job
123 went and where it writes, without being told. Records are kept for 30
days, long enough to read a finished job's output.

## logs and cancel

`aicentre logs <job> [-f] [-n lines] [--err]` reads a job's output from the
file its scheduler writes. It uses the record's file. For other jobs it asks
the scheduler: `bjobs -o output_file` (or `error_file`) on DTU,
`scontrol show job` and its `StdOut=`/`StdErr=` on Gefion. `-f` runs
`tail -F` on the cluster. Ctrl-C stops following, and the job keeps running.

`aicentre cancel <job...>` ends jobs, batch or interactive, with `bkill` or
`scancel`, then drops their records and notes. It prints one line per
provider, plus what bkill said per job. scancel says nothing.

Both find the provider in this order (`src/batch/which.ts`): `-p`, the
submit record, a GPU session's note (those run on DTU), then the one provider
this machine uses when there is only one. Otherwise they ask for `-p`.

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
`Job` shape (`src/jobs/types.ts`): state running, pending, suspended or other,
the scheduler's own state word, queue, name, first node, time used and time
left as `h:mm:ss` or `d-hh:mm:ss`, and the pending reason. Running jobs come
first.

The parsers handle the schedulers' own formats. bjobs prints `exec_host` as
`4*n-62-30-1:2*n-62-30-2`, `run_time` as `300 second(s)` and `time_left` as
`0:55 L` (hours and minutes, then a letter for the kind of limit). squeue
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
