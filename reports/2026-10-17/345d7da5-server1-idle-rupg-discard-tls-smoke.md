# idle-rupg-discard-tls on server1, 2026-10-17

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| unix_time | 1792267091 |
| kernel | 6.8.0-101-generic |
| suite | idle-rupg-discard-tls |
| machine | server1 |
| date | 2026-10-17 |
| commit | 345d7da5 |
| smoke | true |
| server | pid 826183 and the processes below it |
| variant | discard |
| transport | tls |

## idle

| Field | Value |
|---|---|
| server_version | 19.0 (rupg 0.1.40) |
| sessions | 1000 |
| pss_before_bytes | 33361920 |
| pss_after_bytes | 88972288 |
| pss_per_session_bytes | 55610 |
| memory_current_before_bytes | null |
| memory_current_after_bytes | null |
| memory_current_per_session_bytes | null |
| limit_bytes | 65536 |
| passes | true |

