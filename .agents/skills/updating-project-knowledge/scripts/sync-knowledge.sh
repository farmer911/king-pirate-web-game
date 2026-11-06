#!/usr/bin/env bash
set -euo pipefail

cd "$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"

"${GRAPHIFY_PYTHON:-python3}" -c '
import json
from pathlib import Path
from graphify.detect import detect, save_manifest
from graphify.extract import collect_files, extract
from graphify.build import build_from_json
from graphify.cluster import cluster, score_all
from graphify.analyze import god_nodes, surprising_connections, suggest_questions
from graphify.report import generate
from graphify.export import to_json, to_html, to_obsidian

Path("graphify-out").mkdir(exist_ok=True)
detection = detect(Path("."))
Path("graphify-out/.graphify_detect.json").write_text(json.dumps(detection, indent=2))

code_files = []
for f in detection.get("files", {}).get("code", []):
    if f.endswith("battle-dwarf-types.h"):
        continue
    p = Path(f)
    code_files.extend(collect_files(p) if p.is_dir() else [p])

ast_result = extract(code_files) if code_files else {"nodes": [], "edges": [], "hyperedges": [], "input_tokens": 0, "output_tokens": 0}

semantic_nodes = [
    {"id": "doc_gemini_md", "label": "GEMINI.md (100% Rust & Zero-Hallucination SSoT)", "file_type": "document", "source_file": "GEMINI.md", "source_location": "L1"},
    {"id": "doc_rewrite_plan", "label": "REWRITE_PLAN.md (100% Rust & Anti-Bloat Plan)", "file_type": "document", "source_file": "REWRITE_PLAN.md", "source_location": "L1"},
    {"id": "doc_state_md", "label": "STATE.md (Rust Investigation & Phase Tracker)", "file_type": "document", "source_file": ".agents/STATE.md", "source_location": "L1"},
    {"id": "doc_archify_html", "label": "docs/rewrite-plan.html (Interactive Archify Plan)", "file_type": "document", "source_file": "docs/rewrite-plan.html", "source_location": "L1"},
    {"id": "rule_core", "label": "Core Rules (00-core.md, the only always-on rule)", "file_type": "document", "source_file": ".agents/rules/00-core.md", "source_location": "L1"},
    {"id": "doc_claude_md", "label": "CLAUDE.md (imports GEMINI.md and core rules)", "file_type": "document", "source_file": "CLAUDE.md", "source_location": "L1"},
    {"id": "skill_porting_rust", "label": "Skill: porting-legacy-to-rust", "file_type": "document", "source_file": ".agents/skills/porting-legacy-to-rust/SKILL.md", "source_location": "L1"},
    {"id": "skill_updating_knowledge", "label": "Skill: updating-project-knowledge", "file_type": "document", "source_file": ".agents/skills/updating-project-knowledge/SKILL.md", "source_location": "L1"},
    {"id": "skill_planning_archify", "label": "Skill: planning-with-archify", "file_type": "document", "source_file": ".agents/skills/planning-with-archify/SKILL.md", "source_location": "L1"},
    {"id": "rust_protocol_mod", "label": "Rust src/protocol.rs (8-Byte Framing & XOR Checksum)", "file_type": "code", "source_file": "src/protocol.rs", "source_location": "L1"},
    {"id": "rust_main_bin", "label": "Rust src/main.rs (king-pirate Single Binary)", "file_type": "code", "source_file": "src/main.rs", "source_location": "L1"},
    {"id": "daemon_lcserver", "label": "lcserver C++ Gateway (TCP :8002 -> FastCGI :5555)", "file_type": "document", "source_file": "legacy-ref/lcserver/conf/game20002.xml", "source_location": "L14"},
    {"id": "daemon_phpproxy", "label": "phpproxy C++ Router (phpproxy.sock -> TCP :3300/:1234/:8002)", "file_type": "document", "source_file": "legacy-ref/phpproxy/conf/module.xml", "source_location": "L1"},
    {"id": "daemon_dataproxy", "label": "dataproxy C++ DB/Cache Proxy (TCP :3300 -> memcached/MySQL)", "file_type": "document", "source_file": "legacy-ref/dataproxy/conf/config.xml", "source_location": "L16"},
    {"id": "daemon_battle", "label": "battle C++/Lua Combat Engine (TCP :1234, DWARF unstripped)", "file_type": "document", "source_file": "legacy-ref/battle/conf/config.xml", "source_location": "L18"},
    {"id": "schema_pirate20002", "label": "MySQL pirate20002 Schema (210 Tables, Sharded t_bag/t_battle_record)", "file_type": "document", "source_file": "legacy-ref/schema/pirate20002-schema.sql", "source_location": "L1"}
]

semantic_edges = [
    {"source": "doc_gemini_md", "target": "rule_core", "relation": "enforces", "confidence": "EXTRACTED", "source_file": "GEMINI.md"},
    {"source": "doc_claude_md", "target": "doc_gemini_md", "relation": "imports", "confidence": "EXTRACTED", "source_file": "CLAUDE.md"},
    {"source": "rule_core", "target": "skill_porting_rust", "relation": "routes_to", "confidence": "EXTRACTED", "source_file": ".agents/rules/00-core.md"},
    {"source": "rule_core", "target": "skill_updating_knowledge", "relation": "routes_to", "confidence": "EXTRACTED", "source_file": ".agents/rules/00-core.md"},
    {"source": "rule_core", "target": "skill_planning_archify", "relation": "routes_to", "confidence": "EXTRACTED", "source_file": ".agents/rules/00-core.md"},
    {"source": "skill_planning_archify", "target": "doc_archify_html", "relation": "generates", "confidence": "EXTRACTED", "source_file": ".agents/skills/planning-with-archify/SKILL.md"},
    {"source": "skill_planning_archify", "target": "doc_rewrite_plan", "relation": "updates", "confidence": "EXTRACTED", "source_file": ".agents/skills/planning-with-archify/SKILL.md"},
    {"source": "doc_rewrite_plan", "target": "doc_state_md", "relation": "tracks_in", "confidence": "EXTRACTED", "source_file": "REWRITE_PLAN.md"},
    {"source": "rust_main_bin", "target": "rust_protocol_mod", "relation": "uses", "confidence": "EXTRACTED", "source_file": "src/main.rs"},
    {"source": "rust_main_bin", "target": "daemon_lcserver", "relation": "replaces", "confidence": "EXTRACTED", "source_file": "REWRITE_PLAN.md"},
    {"source": "rust_main_bin", "target": "daemon_phpproxy", "relation": "replaces", "confidence": "EXTRACTED", "source_file": "REWRITE_PLAN.md"},
    {"source": "rust_main_bin", "target": "daemon_dataproxy", "relation": "replaces", "confidence": "EXTRACTED", "source_file": "REWRITE_PLAN.md"},
    {"source": "rust_main_bin", "target": "daemon_battle", "relation": "replaces", "confidence": "EXTRACTED", "source_file": "REWRITE_PLAN.md"},
    {"source": "daemon_lcserver", "target": "daemon_phpproxy", "relation": "communicates_via", "confidence": "EXTRACTED", "source_file": "REWRITE_PLAN.md"},
    {"source": "daemon_phpproxy", "target": "daemon_dataproxy", "relation": "routes_amf3_to", "confidence": "EXTRACTED", "source_file": "legacy-ref/phpproxy/conf/module.xml"},
    {"source": "daemon_phpproxy", "target": "daemon_battle", "relation": "routes_amf3_to", "confidence": "EXTRACTED", "source_file": "legacy-ref/phpproxy/conf/module.xml"},
    {"source": "daemon_dataproxy", "target": "schema_pirate20002", "relation": "queries_and_caches", "confidence": "EXTRACTED", "source_file": "legacy-ref/dataproxy/conf/config.xml"}
]

seen_ids = {n["id"] for n in ast_result["nodes"]}
merged_nodes = list(ast_result["nodes"])
for n in semantic_nodes:
    if n["id"] not in seen_ids:
        merged_nodes.append(n)
        seen_ids.add(n["id"])

node_by_label = {n.get("label"): n["id"] for n in merged_nodes}
for cls_name, daemon_id, rel in [
    ("RPCFramework", "daemon_lcserver", "invoked_via_fastcgi_by"),
    ("PHPProxy", "rust_protocol_mod", "ported_to_rust_in"),
    ("CData", "daemon_dataproxy", "sends_query_ast_to"),
    ("BattleLogic", "daemon_battle", "calls_doHero_on"),
    ("ServerProxy", "daemon_lcserver", "sends_private_rpc_to"),
    ("BtstoreElement", "doc_rewrite_plan", "identified_flock_bottleneck_in"),
    ("BabelCrypt", "rust_protocol_mod", "ported_to_rust_in"),
    ("CHero", "daemon_battle", "combat_unit_in"),
    ("CSkill", "daemon_battle", "skill_executor_in"),
    ("CFormation", "daemon_battle", "matrix_formation_in"),
    ("CBuffer", "daemon_battle", "buff_state_in"),
]:
    if cls_name in node_by_label:
        semantic_edges.append({"source": node_by_label[cls_name], "target": daemon_id, "relation": rel, "confidence": "EXTRACTED", "source_file": "REWRITE_PLAN.md"})

merged_edges = list(ast_result["edges"]) + semantic_edges
merged = {
    "nodes": merged_nodes,
    "edges": merged_edges,
    "hyperedges": ast_result.get("hyperedges", []),
    "input_tokens": 0,
    "output_tokens": 0,
}
Path("graphify-out/.graphify_extract.json").write_text(json.dumps(merged, indent=2))

G = build_from_json(merged)
communities = cluster(G)
cohesion = score_all(G, communities)
tokens = {"input": 0, "output": 0}
gods = god_nodes(G)
surprises = surprising_connections(G, communities)
labels = {cid: "Community " + str(cid) for cid in communities}
questions = suggest_questions(G, communities, labels)

report = generate(G, communities, cohesion, labels, gods, surprises, detection, tokens, ".", suggested_questions=questions)
Path("graphify-out/GRAPH_REPORT.md").write_text(report)
Path("graphify-out/graph.json").unlink(missing_ok=True)
to_json(G, communities, "graphify-out/graph.json")
to_html(G, communities, "graphify-out/graph.html", community_labels=labels or None)
note_count = to_obsidian(G, communities, Path("graphify-out/obsidian"), community_labels=labels or None, cohesion=cohesion)
save_manifest(detection["files"])
print(f"[sync-knowledge] OK: {G.number_of_nodes()} nodes, {G.number_of_edges()} edges, {len(communities)} communities, {note_count} Obsidian notes")
'
