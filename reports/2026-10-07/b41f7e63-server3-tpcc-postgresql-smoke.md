# tpcc-postgresql on server3, 2026-10-07

**This is a smoke run.** It shows that the driver works. It ran on a small scale or on a shared machine, so its numbers are not baselines and no claim may use them.

| Field | Value |
|---|---|
| suite | tpcc-postgresql |
| machine | server3 |
| date | 2026-10-07 |
| commit | b41f7e63 |
| smoke | true |
| note | smoke run: it shows that the driver works, it is not a baseline |
| engine_name | postgresql |
| cgroup | /sys/fs/cgroup/system.slice/system-postgresql.slice/postgresql@19-ycsb.service |
| hammerdb_version | 6.0 |
| rampup_min | 1 |
| duration_min | 1 |
| seed | 1 |
| mix | HammerDB: New-Order 10/23, Payment 10/23, Delivery, Stock-Level and Order-Status 1/23 each, no keying or think time |
| kernel | 6.8.0-106-generic |
| server_version | 19beta4 |
| warehouses | 2 |
| server_cores | 8 |

## build

| Field | Value |
|---|---|
| warehouses | 2 |
| virtual_users | 2 |
| secs | 298.9 |
| database_bytes | 227910159 |
| server.wall_s | 298.905259 |
| server.cpu_usec | 29346675 |
| server.user_usec | 22771921 |
| server.system_usec | 6574755 |
| server.rbytes | 9531392 |
| server.wbytes | 480669696 |
| server.memory_peak_bytes | 665829376 |
| server.memory_peak_scope | since the cgroup was created |
| server.file_max_bytes | 607207424 |
| server.pss_max_bytes | 212278272 |
| server.memory_current_max_bytes | 665518080 |
| server.samples | 1002 |

## check_before

| condition | text | checked | failed |
|---|---|---|---|
| 1 | W_YTD = sum(D_YTD) for each warehouse | 2 | 0 |
| 2 |  | 20 | 0 |
| 3 |  | 20 | 0 |
| 4 | sum(O_OL_CNT) = rows in ORDER-LINE for each district | 20 | 0 |

## fdatasync

| Field | Value |
|---|---|
| dir | /var/lib/postgresql/19 |
| rounds | 200 |
| write_bytes | 8192 |
| p50_us | 1674.7 |

## idle_base

| Field | Value |
|---|---|
| wall_s | 10.009195 |
| cpu_usec | 119402 |
| user_usec | 41675 |
| system_usec | 77728 |
| rbytes | 0 |
| wbytes | 3547136 |
| memory_peak_bytes | 665829376 |
| memory_peak_scope | since the cgroup was created |
| file_max_bytes | 611614720 |
| pss_max_bytes | 174551040 |
| memory_current_max_bytes | 643317760 |
| samples | 58 |

## rows

| form | virtual_users | synchronous_commit | synchronous_commit_set_by_driver | tpm_source | time_profile.neword.calls | time_profile.neword.min_ms | time_profile.neword.avg_ms | time_profile.neword.max_ms | time_profile.neword.total_ms | time_profile.neword.p99_ms | time_profile.neword.p95_ms | time_profile.neword.p75_ms | time_profile.neword.p50_ms | time_profile.neword.p25_ms | time_profile.neword.sd_ms | time_profile.neword.ratio_percent | time_profile.payment.calls | time_profile.payment.min_ms | time_profile.payment.avg_ms | time_profile.payment.max_ms | time_profile.payment.total_ms | time_profile.payment.p99_ms | time_profile.payment.p95_ms | time_profile.payment.p75_ms | time_profile.payment.p50_ms | time_profile.payment.p25_ms | time_profile.payment.sd_ms | time_profile.payment.ratio_percent | time_profile.slev.calls | time_profile.slev.min_ms | time_profile.slev.avg_ms | time_profile.slev.max_ms | time_profile.slev.total_ms | time_profile.slev.p99_ms | time_profile.slev.p95_ms | time_profile.slev.p75_ms | time_profile.slev.p50_ms | time_profile.slev.p25_ms | time_profile.slev.sd_ms | time_profile.slev.ratio_percent | time_profile.delivery.calls | time_profile.delivery.min_ms | time_profile.delivery.avg_ms | time_profile.delivery.max_ms | time_profile.delivery.total_ms | time_profile.delivery.p99_ms | time_profile.delivery.p95_ms | time_profile.delivery.p75_ms | time_profile.delivery.p50_ms | time_profile.delivery.p25_ms | time_profile.delivery.sd_ms | time_profile.delivery.ratio_percent | time_profile.ostat.calls | time_profile.ostat.min_ms | time_profile.ostat.avg_ms | time_profile.ostat.max_ms | time_profile.ostat.total_ms | time_profile.ostat.p99_ms | time_profile.ostat.p95_ms | time_profile.ostat.p75_ms | time_profile.ostat.p50_ms | time_profile.ostat.p25_ms | time_profile.ostat.sd_ms | time_profile.ostat.ratio_percent | time_profile_note | nopm | tpm | nopm_per_core | server_cpu_usec_per_new_order | server_write_bytes_per_new_order | server_memory_peak | sync_bound_nopm | server.wall_s | server.cpu_usec | server.user_usec | server.system_usec | server.rbytes | server.wbytes | server.memory_peak_bytes | server.memory_peak_scope | server.file_max_bytes | server.pss_max_bytes | server.memory_current_max_bytes | server.samples | check.0.condition | check.0.text | check.0.checked | check.0.failed | check.1.condition | check.1.checked | check.1.failed | check.2.condition | check.2.checked | check.2.failed | check.3.condition | check.3.text | check.3.checked | check.3.failed | latency.neword.count | latency.neword.mean_us | latency.neword.p50_us | latency.neword.p95_us | latency.neword.p99_us | latency.neword.p999_us | latency.neword.max_us | latency.payment.count | latency.payment.mean_us | latency.payment.p50_us | latency.payment.p95_us | latency.payment.p99_us | latency.payment.p999_us | latency.payment.max_us | latency.ostat.count | latency.ostat.mean_us | latency.ostat.p50_us | latency.ostat.p95_us | latency.ostat.p99_us | latency.ostat.p999_us | latency.ostat.max_us | latency.delivery.count | latency.delivery.mean_us | latency.delivery.p50_us | latency.delivery.p95_us | latency.delivery.p99_us | latency.delivery.p999_us | latency.delivery.max_us | latency.slev.count | latency.slev.mean_us | latency.slev.p50_us | latency.slev.p95_us | latency.slev.p99_us | latency.slev.p999_us | latency.slev.max_us | new_order_rollbacks | deliveries_skipped_districts | retries | errors | first_error |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| procedures | 4 | on | true | HammerDB TEST RESULT: pg_stat_database commits and rollbacks | 3745 | 2.114 | 57.281 | 890.381 | 214516.204 | 276.326 | 167.94 | 78.787 | 41.078 | 16.725 | 59.783 | 44.546 | 3735 | 0.935 | 51.736 | 846.372 | 193234.171 | 259.351 | 163.743 | 70.398 | 32.316 | 13.29 | 58.181 | 40.126 | 390 | 2.17 | 110.641 | 678.323 | 43149.914 | 457.944 | 292.36 | 156.867 | 86.413 | 35.935 | 100.594 | 8.96 | 368 | 4.754 | 66.163 | 374.149 | 24348.011 | 287.993 | 199.245 | 83.917 | 50.893 | 20.743 | 62.468 | 5.056 | 402 | 0.532 | 14.584 | 383.284 | 5862.847 | 124.195 | 60.844 | 11.774 | 4.558 | 1.386 | 30.617 | 1.217 | HammerDB times each procedure call over the whole run, the ramp too | 2069 | 4726 | 259 | 8084 | 56912 | 796893184 | 64491 | 60.41031 | 16724964 | 13846313 | 2878651 | 1191936 | 117751808 | 796893184 | since the cgroup was created | 737177600 | 202209280 | 796381184 | 187 | 1 | W_YTD = sum(D_YTD) for each warehouse | 2 | 0 | 2 | 20 | 0 | 3 | 20 | 0 | 4 | sum(O_OL_CNT) = rows in ORDER-LINE for each district | 20 | 0 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| statements | 4 | on | true | the transactions that the terminals finished in the interval |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1094 | 2404 | 137 | 11310 | 59465 | 796893184 | 64491 | 59.945986 | 12406810 | 8426527 | 3980283 | 1654784 | 65232896 | 796893184 | since the cgroup was created | 675782656 | 190953472 | 726523904 | 147 | 1 | W_YTD = sum(D_YTD) for each warehouse | 2 | 0 | 2 | 20 | 0 | 3 | 20 | 0 | 4 | sum(O_OL_CNT) = rows in ORDER-LINE for each district | 20 | 0 | 1102 | 142921.9 | 70778.9 | 490733.6 | 620757 | 1061158.9 | 1384812.8 | 1014 | 52004.1 | 25034.8 | 181403.6 | 327155.7 | 501219.3 | 560985.6 | 86 | 13390.5 | 4816.9 | 63176.7 | 149141.9 | 149141.9 | 149141.9 | 117 | 218326.1 | 149946.4 | 637534.2 | 750780.4 | 1124841.1 | 1124841.1 | 91 | 22687.6 | 10813.4 | 77594.6 | 153501.9 | 153501.9 | 153501.9 | 9 | 16 | 0 | 0 | null |
| procedures | 4 | off | true | HammerDB TEST RESULT: pg_stat_database commits and rollbacks | 6484 | 0.66 | 34.181 | 954.056 | 221628.359 | 250.179 | 112.984 | 46.524 | 14.279 | 5.283 | 53.653 | 45.891 | 6318 | 0.335 | 24.118 | 854.833 | 152379.992 | 191.788 | 89.438 | 28.322 | 9.214 | 3.12 | 41.463 | 31.553 | 641 | 1.591 | 68.629 | 2118.375 | 43991.472 | 563.597 | 258.853 | 74.916 | 27.085 | 6.692 | 142.447 | 9.109 | 671 | 1.685 | 58.793 | 1000.77 | 39450.339 | 357.977 | 200.71 | 75.23 | 31.834 | 10.857 | 83.027 | 8.169 | 634 | 0.299 | 34.293 | 600.456 | 21741.885 | 278.399 | 115.99 | 43.568 | 14.04 | 4.477 | 57.434 | 4.502 | HammerDB times each procedure call over the whole run, the ramp too | 3770 | 8575 | 471 | 6172 | 48765 | 796893184 | null | 60.78919 | 23269569 | 19612017 | 3657552 | 22171648 | 183844864 | 796893184 | since the cgroup was created | 399056896 | 236539904 | 482545664 | 177 | 1 | W_YTD = sum(D_YTD) for each warehouse | 2 | 0 | 2 | 20 | 0 | 3 | 20 | 0 | 4 | sum(O_OL_CNT) = rows in ORDER-LINE for each district | 20 | 0 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| statements | 4 | off | true | the transactions that the terminals finished in the interval |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1109 | 2449 | 139 | 10920 | 35351 | 796893184 | null | 59.973384 | 12121626 | 8961422 | 3160205 | 1486848 | 39239680 | 796893184 | since the cgroup was created | 384950272 | 192243712 | 428191744 | 124 | 1 | W_YTD = sum(D_YTD) for each warehouse | 2 | 0 | 2 | 20 | 0 | 3 | 20 | 0 | 4 | sum(O_OL_CNT) = rows in ORDER-LINE for each district | 20 | 0 | 1119 | 148849 | 51380.2 | 499122.2 | 796917.8 | 1090519 | 1404995.1 | 1026 | 39874.5 | 7995.4 | 159383.6 | 362807.3 | 692060.2 | 821887.2 | 94 | 17036.6 | 3424.3 | 76021.8 | 164448.3 | 164448.3 | 164448.3 | 117 | 225826.6 | 116916.2 | 729808.9 | 977272.8 | 1301864 | 1301864 | 95 | 31930.9 | 14680.1 | 108003.3 | 171355 | 171355 | 171355 | 12 | 17 | 0 | 0 | null |

