//! The gates of `spec/02-the-goal.md` section 2.10 of tamnd/rupg.
//!
//! A gate passes or fails on a measured number. When a gate fails, the report gives the number, the gap and the cause. The target does not change in the same change that reports the miss.

/// One gate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Gate {
    /// G1 to G12.
    pub(crate) id: &'static str,
    /// The milestone that the gate closes.
    pub(crate) milestone: &'static str,
    /// The suite and the machine.
    pub(crate) suite: &'static str,
    /// The number that passes.
    pub(crate) target: &'static str,
}

/// The gates, in the order of the spec. G1 and G12 are decided in tamnd/rupg-compat.
pub(crate) const GATES: [Gate; 12] = [
    Gate {
        id: "G1",
        milestone: "M2",
        suite: "L1 and L2",
        target: "7 clients connect and show the schema",
    },
    Gate {
        id: "G2",
        milestone: "M4",
        suite: "ClickBench, c6a.4xlarge",
        target: "hot sum below ClickHouse (17.53 s, remeasured)",
    },
    Gate {
        id: "G3",
        milestone: "M4",
        suite: "ClickBench, c6a.4xlarge",
        target: "disk below 9.45 GB",
    },
    Gate {
        id: "G4",
        milestone: "M3",
        suite: "YCSB A, B, C, F",
        target: "3x server CPU per operation, no pipelining",
    },
    Gate { id: "G5", milestone: "M5", suite: "TPC-C, statements", target: "3x NOPM per core" },
    Gate { id: "G6", milestone: "M7", suite: "TPC-C, procedures", target: "10x NOPM per core" },
    Gate {
        id: "G7",
        milestone: "M8",
        suite: "ClickBench, c6a.4xlarge",
        target: "hot sum at most 1.753 s, cold sum at most 11.37 s",
    },
    Gate {
        id: "G8",
        milestone: "M8",
        suite: "ClickBench, c6a.4xlarge",
        target: "10x peak memory and CPU time, disk below 7.67 GB",
    },
    Gate { id: "G9", milestone: "M8", suite: "TPC-H SF100", target: "10x DuckDB hot total" },
    Gate {
        id: "G10",
        milestone: "M8",
        suite: "ClickBench, c6a.metal",
        target: "hot sum at most 0.440 s",
    },
    Gate { id: "G11", milestone: "M11", suite: "TPC-C, 2 to 16 nodes", target: "at least 0.8 N" },
    Gate {
        id: "G12",
        milestone: "M12",
        suite: "L1 to L5, 19 and shims 14 to 18",
        target: "100 percent",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn twelve_gates_in_order() {
        let ids: Vec<&str> = GATES.iter().map(|g| g.id).collect();
        let want: Vec<String> = (1..=12).map(|n| format!("G{n}")).collect();
        assert_eq!(ids, want);
    }

    #[test]
    fn the_hot_target_is_a_tenth_of_clickhouse() {
        let clickhouse_hot_sum_s: f64 = 17.53;
        assert!(GATES[6].target.starts_with("hot sum at most 1.753 s"));
        assert!((clickhouse_hot_sum_s / 10.0 - 1.753).abs() < 1e-9);
    }
}
