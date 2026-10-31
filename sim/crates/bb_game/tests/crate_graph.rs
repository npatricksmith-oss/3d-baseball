//! PLAN §15: `bb_agents` must not reach `bb_world`, directly or transitively. This makes the
//! belief/world split of D1 a build-time rule; this test keeps a manifest edit from undoing it.

use std::collections::{BTreeMap, BTreeSet};
use std::process::Command;

fn workspace_graph() -> BTreeMap<String, Vec<String>> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let out = Command::new(cargo)
        .args(["metadata", "--format-version", "1", "--no-deps", "--offline"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run cargo metadata");
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let mut g = BTreeMap::new();
    for p in v["packages"].as_array().unwrap() {
        let deps = p["dependencies"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|d| d["kind"].is_null()) // normal deps only
            .map(|d| d["name"].as_str().unwrap().to_string())
            .collect();
        g.insert(p["name"].as_str().unwrap().to_string(), deps);
    }
    g
}

fn reachable(g: &BTreeMap<String, Vec<String>>, from: &str) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut stack = vec![from.to_string()];
    while let Some(n) = stack.pop() {
        for d in g.get(&n).into_iter().flatten() {
            if seen.insert(d.clone()) {
                stack.push(d.clone());
            }
        }
    }
    seen
}

#[test]
fn agents_cannot_reach_world() {
    let g = workspace_graph();
    for c in ["bb_agents", "bb_observe", "bb_physics", "bb_math"] {
        let r = reachable(&g, c);
        assert!(!r.contains("bb_world"), "{c} reaches bb_world via {r:?}");
        assert!(!r.contains("bb_game"), "{c} reaches bb_game via {r:?}");
    }
}

#[test]
fn physics_is_pure() {
    // bb_physics depends only on bb_math (and libm): no agents, rules, or game state.
    let g = workspace_graph();
    let r = reachable(&g, "bb_physics");
    for bad in ["bb_agents", "bb_observe", "bb_rules", "bb_world", "bb_schema"] {
        assert!(!r.contains(bad), "bb_physics reaches {bad}");
    }
}
