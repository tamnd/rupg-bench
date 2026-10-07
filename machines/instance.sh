#!/bin/bash
# Start a machine of spec/20 section 20.2 on AWS, run one suite on it with run-suite.sh, copy the reports back and stop the machine.
#
# Usage: instance.sh NAME SUITE [KEY=VALUE...]. NAME is 4xl, metal, tpch or oltp. SUITE and the KEY=VALUE settings go to machines/run-suite.sh on the machine, for example SMOKE=1 or SCALES="1 10".
#
#   4xl    c6a.4xlarge, 16 vCPU, 32 GiB, 500 GB gp2, as the ClickBench pin
#   metal  c6a.metal, 192 vCPU, 384 GiB, 500 GB gp2
#   tpch   c6a.4xlarge for SF1 and SF10. SF100 needs at least 64 vCPU, 256 GiB and local NVMe, so set INSTANCE_TYPE.
#   oltp   not chosen yet (spec/20 section 20.2), so set INSTANCE_TYPE, or HOST for a machine that is not on AWS.
#
# The AWS settings come from the environment: AWS_REGION, KEY_NAME (the key pair), SSH_KEY (its private key file), SECURITY_GROUP (it must let ssh in), and SUBNET when the account has no default subnet. AMI is Ubuntu 24.04 by default. VOLUME_SIZE and VOLUME_TYPE change the root volume.
#
# HOST=user@host runs on a machine that exists, with no AWS call, and does not stop it. MACHINE sets the machine name of the reports (NAME by default). REMOTE_DIR is the copy of the repository on the machine (rupg-bench in the home directory). SSH_OPTS adds ssh options. DRY_RUN=1 prints the AWS command and stops.
#
# The machine is terminated when the script exits for any reason, after the reports and the log are copied back. The log goes to runs/tmp/.
set -euo pipefail
# shellcheck source=machines/lib.sh
. "$(dirname "$0")/lib.sh"

[ $# -ge 2 ] || die "usage: instance.sh NAME SUITE [KEY=VALUE...]"
name=$1
suite=$2
shift 2
settings=()
for kv in "$@"; do
    [[ $kv =~ ^[A-Z_][A-Z0-9_]*= ]] || die "the setting $kv is not KEY=VALUE"
    settings+=("$kv")
done

volume_type=${VOLUME_TYPE:-gp2}
volume_size=${VOLUME_SIZE:-500}
case $name in
4xl) type=c6a.4xlarge ;;
metal) type=c6a.metal ;;
tpch) type=${INSTANCE_TYPE:-c6a.4xlarge} ;;
oltp)
    [ -n "${INSTANCE_TYPE:-}${HOST:-}" ] || die "the oltp machine is not chosen yet: set INSTANCE_TYPE or HOST"
    type=${INSTANCE_TYPE:-}
    ;;
*) die "unknown machine $name: use 4xl, metal, tpch or oltp" ;;
esac
machine=${MACHINE:-$name}
remote_dir=${REMOTE_DIR:-rupg-bench}
stamp=$(date -u +%Y%m%dT%H%M%SZ)
mkdir -p "$RUPG_BENCH_ROOT/runs/tmp"
logfile=$RUPG_BENCH_ROOT/runs/tmp/$stamp-$machine-$suite.log
# shellcheck disable=SC2206
ssh_opts=(-o BatchMode=yes -o ConnectTimeout=10 ${SSH_OPTS:-})
instance=""

launch() {
    local ami subnet=() out
    for v in AWS_REGION KEY_NAME SSH_KEY SECURITY_GROUP; do
        [ -n "${!v:-}" ] || die "set $v, or set HOST to use a machine that exists"
    done
    ami=${AMI:-resolve:ssm:/aws/service/canonical/ubuntu/server/24.04/stable/current/amd64/hvm/ebs-gp3/ami-id}
    [ -n "${SUBNET:-}" ] && subnet=(--subnet-id "$SUBNET")
    local cmd=(aws ec2 run-instances --region "$AWS_REGION" --image-id "$ami" --instance-type "$type"
        --key-name "$KEY_NAME" --security-group-ids "$SECURITY_GROUP" ${subnet[@]+"${subnet[@]}"}
        --block-device-mappings "[{\"DeviceName\":\"/dev/sda1\",\"Ebs\":{\"VolumeSize\":$volume_size,\"VolumeType\":\"$volume_type\",\"DeleteOnTermination\":true}}]"
        --instance-initiated-shutdown-behavior terminate
        --tag-specifications "ResourceType=instance,Tags=[{Key=Name,Value=rupg-bench-$machine-$suite}]"
        --query 'Instances[0].InstanceId' --output text)
    if [ "${DRY_RUN:-0}" = 1 ]; then
        printf '%q ' "${cmd[@]}"
        printf '\n'
        exit 0
    fi
    out=$("${cmd[@]}")
    instance=$out
    log "started $instance ($type) for $machine"
    aws ec2 wait instance-running --region "$AWS_REGION" --instance-ids "$instance"
    local ip
    ip=$(aws ec2 describe-instances --region "$AWS_REGION" --instance-ids "$instance" \
        --query 'Reservations[0].Instances[0].PublicIpAddress' --output text)
    HOST=ubuntu@$ip
    ssh_opts+=(-i "$SSH_KEY" -o StrictHostKeyChecking=accept-new)
}

finish() {
    local status=$?
    trap - EXIT
    if [ -n "${HOST:-}" ]; then
        log "copy the reports back from $HOST"
        rsync -az -e "ssh ${ssh_opts[*]}" "$HOST:$remote_dir/reports/" "$RUPG_BENCH_ROOT/reports/" ||
            log "warning: the copy of the reports failed"
    fi
    if [ -n "$instance" ]; then
        log "terminate $instance"
        aws ec2 terminate-instances --region "$AWS_REGION" --instance-ids "$instance" >/dev/null
        aws ec2 wait instance-terminated --region "$AWS_REGION" --instance-ids "$instance" ||
            log "warning: $instance did not reach terminated, check the console"
    fi
    log "the log is $logfile"
    exit "$status"
}

on_host() {
    # shellcheck disable=SC2029
    ssh "${ssh_opts[@]}" "$HOST" "$@"
}

if [ -z "${HOST:-}" ]; then
    launch
fi
trap finish EXIT
log "wait for ssh on $HOST"
for i in $(seq 60); do
    on_host true 2>/dev/null && break
    [ "$i" -lt 60 ] || die "no ssh on $HOST after 5 minutes"
    sleep 5
done
{
    printf 'machine %s, instance %s, type %s, suite %s, started %s\n' "$machine" "${instance:-none}" "${type:-none}" "$suite" "$stamp"
    on_host 'uname -a; lscpu | grep -E "^(Model name|CPU\(s\)|Thread|NUMA node\(s\))"; free -g | head -2; lsblk -d -o NAME,SIZE,ROTA,MODEL'
} 2>&1 | tee "$logfile"
log "copy the repository to $HOST:$remote_dir"
on_host "mkdir -p $(printf '%q' "$remote_dir")"
rsync -az --delete --exclude /target --exclude /reports --exclude /runs -e "ssh ${ssh_opts[*]}" \
    "$RUPG_BENCH_ROOT/" "$HOST:$remote_dir/"
remote="cd $(printf '%q' "$remote_dir") && env"
for kv in ${settings[@]+"${settings[@]}"}; do
    remote+=" $(printf '%q' "$kv")"
done
remote+=" machines/run-suite.sh $(printf '%q' "$suite") $(printf '%q' "$machine")"
log "run $suite on $HOST"
on_host "$remote" 2>&1 | tee -a "$logfile"
