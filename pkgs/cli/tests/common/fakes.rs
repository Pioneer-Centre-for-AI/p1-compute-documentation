pub const FAKE_SSH: &str = r##"#!/bin/sh
A="$FAKE/answers"
echo "$*" >> "$FAKE/calls"

case " $* " in
  *" ControlMaster=yes "*) touch "$FAKE/master.up"; exit 0 ;;
  *" -O check "*) [ -f "$FAKE/master.up" ]; exit $? ;;
  *" -O cancel "*) echo "$*" >> "$FAKE/cancel"; exit 0 ;;
  *" -O forward "*) exit 0 ;;
  *" ControlMaster=no -W "*) head -c 64 > "$FAKE/proxy-stdin"; exit 0 ;;
  *" -O exit "*) rm -f "$FAKE/master.up"; exit 0 ;;
esac
for last; do :; done
rec() { printf '%s\n' "$last" >> "$FAKE/$1"; }
hold() { if [ -f "$A/$1" ]; then trap 'exit 143' TERM; while :; do sleep 0.1; done; fi; }
case "$last" in
  *"@@KIND"*) rec kind; if [ -f "$A/kind" ]; then cat "$A/kind"; else echo '@@KIND dir'; fi; exit 0 ;;
  *"AICENTRE_VERSION="*"@@LAUNCHES"*)
    rec install; cat > /dev/null
    if [ -f "$A/install.out" ]; then cat "$A/install.out"; else echo '@@LAUNCHES 0.1.0'; fi
    exit 0 ;;
  *"base64 -d"*) rec push; cat > "$FAKE/pushed"; cat "$A/push.out" 2>/dev/null; exit 0 ;;
  *"@@SUM"*) cat "$A/sum.out" 2>/dev/null; exit 0 ;;
  *"sync.lock"*) rec sync; exit 0 ;;
  *"@@USER"*)
    printf '@@USER s123456\n@@HOME /home/s123456\n'
    printf '@@AICENTRE %s\n@@BINPATH yes\n@@SHELL bash\n' "$(cat "$A/there" 2>/dev/null)"
    [ -f "$A/no-p1" ] || echo '@@P1 yes'
    echo '@@LAUNCHER /home/s123456/.local/share/aicentre/versions/0.1.0/aicentre'
    if [ -f "$A/launches" ]; then echo "@@LAUNCHES $(cat "$A/launches")"; else echo '@@LAUNCHES 0.1.0'; fi
    exit 0 ;;
  *" unpack-example "*) rec example; cat "$A/example.err" >&2 2>/dev/null; exit "$(cat "$A/example.code" 2>/dev/null || echo 0)" ;;
  *"bkill"*) rec bkill; cat "$A/bkill.out" 2>/dev/null || echo "Job <123> is being terminated"; exit "$(cat "$A/bkill.code" 2>/dev/null || echo 0)" ;;
  *"bjobs"*)
    if [ -f "$A/needs-login" ] && [ ! -f "$FAKE/master.up" ]; then echo "s123456@login9.hpc.dtu.dk: Permission denied (password)." >&2; exit 255; fi
    cat "$A/bjobs" 2>/dev/null; cat "$A/bjobs-stderr" >&2 2>/dev/null; exit "$(cat "$A/bjobs.code" 2>/dev/null || echo 0)" ;;
  *"bhosts"*) cat "$A/bhosts" 2>/dev/null; exit 0 ;;
  *"sinfo"*) cat "$A/sinfo" 2>/dev/null; exit 0 ;;
  *"squeue"*) cat "$A/squeue" 2>/dev/null; cat "$A/squeue-stderr" >&2 2>/dev/null; exit "$(cat "$A/squeue.code" 2>/dev/null || echo 0)" ;;
  *"bsub"*)
    rec bsub
    cat "$A/bsub.out" 2>/dev/null
    hold bsub.hold
    exit "$(cat "$A/bsub.code" 2>/dev/null || echo 0)" ;;
  *" launch --login --needs "*) rec needs; cat "$A/needs.out" 2>/dev/null; exit 0 ;;
  *"/aicentre launch --login "*)
    rec login
    cat "$A/login.out" 2>/dev/null
    hold login.hold
    exit "$(cat "$A/login.code" 2>/dev/null || echo 0)" ;;
  *"mkdir -p "*) rec mkdir; exit 0 ;;
  "kill "*) rec kill; exit 0 ;;
esac

case "$1" in
  -V) cat "$A/ssh-V" >&2 2>/dev/null || echo "OpenSSH_9.8p1, LibreSSL 3.3.6" >&2; exit 0 ;;
  -Q) cat "$A/ssh-Q" 2>/dev/null || printf 'ssh-ed25519\nssh-rsa\n'; exit 0 ;;
  -G)
    if [ -f "$A/ssh-G-error" ]; then cat "$A/ssh-G-error" >&2; exit 255; fi
    if [ -f "$A/ssh-G-$2" ]; then cat "$A/ssh-G-$2"; exit 0; fi
    printf 'host %s\nhostname %s\nuser tester\nport 22\nidentityfile ~/.ssh/id_ed25519\nuserknownhostsfile %s/.ssh/known_hosts\nglobalknownhostsfile /etc/ssh/ssh_known_hosts\n' "$2" "$2" "$HOME"
    exit 0 ;;
esac
for last; do :; done
for arg; do [ "$arg" = "$last" ] && break; alias="$arg"; done
cat "$A/probe-$alias.stdout" 2>/dev/null
cat "$A/probe-$alias.stderr" >&2 2>/dev/null
exit "$(cat "$A/probe-$alias.code" 2>/dev/null || echo 255)"
"##;

pub const FAKE_SSH_KEYGEN: &str = r##"#!/bin/sh
A="$FAKE/answers"
case "$1" in
  -F) [ -f "$A/known-$2" ] && { echo "# Host $2 found"; exit 0; }; exit 1 ;;
  -L) [ -f "$A/cert-valid" ] || exit 255; echo "$3:"; echo "        Valid: $(cat "$A/cert-valid")"; exit 0 ;;
esac
exit 1
"##;

pub const FAKE_SSH_ADD: &str = r##"#!/bin/sh
[ -f "$FAKE/answers/agent" ] && { cat "$FAKE/answers/agent"; exit 0; }
echo "The agent has no identities." ; exit 1
"##;

pub const FAKE_TSH: &str = r##"#!/bin/sh
A="$FAKE/answers"
case "$1 $2" in
  "version --client") echo '{"version":"18.10.0"}'; exit 0 ;;
  "status --client") cat "$A/tsh-status" 2>/dev/null || echo "ERROR: Not logged in." >&2; exit 0 ;;
esac
exit 1
"##;
