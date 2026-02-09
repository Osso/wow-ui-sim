//! Code generation (PHF map writers) for trait data.

use super::csv_util::escape_str;
use super::gen_traits_load::*;
use std::fs::File;
use std::io::Write;

// ── Denormalized output structs ──

pub struct TreeInfo {
    pub id: u32,
    pub first_node_id: u32,
    pub flags: u32,
    pub node_ids: Vec<u32>,
    pub currency_ids: Vec<u32>,
}

pub struct NodeInfo {
    pub id: u32,
    pub tree_id: u32,
    pub pos_x: i32,
    pub pos_y: i32,
    pub node_type: u32,
    pub flags: u32,
    pub sub_tree_id: u32,
    pub entry_ids: Vec<u32>,
    pub cond_ids: Vec<u32>,
    pub edges: Vec<(u32, u32, u32)>, // (source_node_id, edge_type, visual_style)
    pub group_ids: Vec<u32>,
}

// ── Denormalization ──

pub fn denormalize(data: &AllData) -> (Vec<TreeInfo>, Vec<NodeInfo>) {
    let nodes_by_tree = group_nodes_by_tree(&data.nodes);
    let edges_by_target = group_edges_by_target(&data.edges);

    let tree_infos: Vec<TreeInfo> = data.trees.iter().map(|t| {
        let node_ids = nodes_by_tree.get(&t.id).cloned().unwrap_or_default();
        let currency_ids = data.joins.tree_to_currencies.get(&t.id).cloned().unwrap_or_default();
        TreeInfo { id: t.id, first_node_id: t.first_node_id, flags: t.flags, node_ids, currency_ids }
    }).collect();

    let node_infos: Vec<NodeInfo> = data.nodes.iter().map(|n| {
        build_node_info(n, &data.joins, &edges_by_target)
    }).collect();

    (tree_infos, node_infos)
}

fn build_node_info(
    n: &RawNode,
    joins: &JoinMaps,
    edges_by_target: &std::collections::HashMap<u32, Vec<(u32, u32, u32)>>,
) -> NodeInfo {
    let entry_ids = joins.node_to_entries.get(&n.id).cloned().unwrap_or_default();
    let cond_ids = joins.node_to_conds.get(&n.id).cloned().unwrap_or_default();
    let edges = edges_by_target.get(&n.id).cloned().unwrap_or_default();
    let group_ids = joins.node_to_groups.get(&n.id).cloned().unwrap_or_default();
    NodeInfo {
        id: n.id, tree_id: n.tree_id, pos_x: n.pos_x, pos_y: n.pos_y,
        node_type: n.node_type, flags: n.flags, sub_tree_id: n.sub_tree_id,
        entry_ids, cond_ids, edges, group_ids,
    }
}

fn group_nodes_by_tree(nodes: &[RawNode]) -> std::collections::HashMap<u32, Vec<u32>> {
    let mut map = std::collections::HashMap::new();
    for n in nodes {
        map.entry(n.tree_id).or_insert_with(Vec::new).push(n.id);
    }
    map
}

fn group_edges_by_target(edges: &[RawEdge]) -> std::collections::HashMap<u32, Vec<(u32, u32, u32)>> {
    let mut map = std::collections::HashMap::new();
    for e in edges {
        // LeftTraitNodeID = prerequisite (source), RightTraitNodeID = dependent (target)
        map.entry(e.right_node_id).or_insert_with(Vec::new).push((
            e.left_node_id, e.edge_type, e.visual_style,
        ));
    }
    map
}

// ── Header and struct definitions ──

pub fn write_header(out: &mut File) -> std::io::Result<()> {
    writeln!(out, "//! Auto-generated trait/talent data from WoW CSV exports.")?;
    writeln!(out, "//! Do not edit manually - regenerate with: wow-cli generate traits")?;
    writeln!(out)?;
    Ok(())
}

pub fn write_struct_defs(out: &mut File) -> std::io::Result<()> {
    write_struct(out, "TraitTreeInfo", &[
        "pub id: u32", "pub first_node_id: u32", "pub flags: u32",
        "pub node_ids: &'static [u32]", "pub currency_ids: &'static [u32]",
    ])?;
    write_struct(out, "TraitNodeInfo", &[
        "pub id: u32", "pub tree_id: u32", "pub pos_x: i32", "pub pos_y: i32",
        "pub node_type: u32", "pub flags: u32", "pub sub_tree_id: u32",
        "pub entry_ids: &'static [u32]", "pub cond_ids: &'static [u32]",
        "pub edges: &'static [TraitEdgeInfo]", "pub group_ids: &'static [u32]",
    ])?;
    write_struct(out, "TraitEntryInfo", &[
        "pub id: u32", "pub definition_id: u32", "pub max_ranks: u32",
        "pub entry_type: u32", "pub sub_tree_id: u32",
    ])?;
    write_struct(out, "TraitDefInfo", &[
        "pub id: u32", "pub spell_id: u32", "pub override_icon: u32",
        "pub overrides_spell_id: u32", "pub visible_spell_id: u32",
        "pub override_name: &'static str", "pub override_subtext: &'static str",
        "pub override_description: &'static str",
    ])?;
    write_struct(out, "TraitCondInfo", &[
        "pub id: u32", "pub cond_type: u32", "pub tree_id: u32",
        "pub granted_ranks: u32", "pub quest_id: u32", "pub achievement_id: u32",
        "pub spec_set_id: u32", "pub group_id: u32", "pub node_id: u32",
        "pub entry_id: u32", "pub currency_id: u32", "pub spent_amount: u32",
        "pub flags: u32", "pub required_level: u32",
    ])?;
    write_struct(out, "TraitSubTreeInfo", &[
        "pub id: u32", "pub name: &'static str", "pub description: &'static str",
        "pub atlas_element_id: u32", "pub tree_id: u32",
    ])?;
    write_struct(out, "TraitCurrencyInfo", &[
        "pub id: u32", "pub currency_type: u32", "pub flags: u32",
    ])?;
    write_struct(out, "TraitEdgeInfo", &[
        "pub source_node_id: u32", "pub edge_type: u32", "pub visual_style: u32",
    ])?;
    Ok(())
}

fn write_struct(out: &mut File, name: &str, fields: &[&str]) -> std::io::Result<()> {
    writeln!(out, "#[derive(Debug, Clone)]")?;
    writeln!(out, "pub struct {} {{", name)?;
    for f in fields {
        writeln!(out, "    {},", f)?;
    }
    writeln!(out, "}}")?;
    writeln!(out)?;
    Ok(())
}

// ── PHF map writers ──

pub fn write_tree_db(
    out: &mut File,
    trees: &[TreeInfo],
) -> Result<(), Box<dyn std::error::Error>> {
    for t in trees {
        emit_u32_array(out, &format!("TREE_{}_NODES", t.id), &t.node_ids)?;
        emit_u32_array(out, &format!("TREE_{}_CURRENCIES", t.id), &t.currency_ids)?;
    }
    let mut builder = phf_codegen::Map::new();
    for t in trees {
        let value = format!(
            "TraitTreeInfo {{ id: {}, first_node_id: {}, flags: {}, \
             node_ids: &TREE_{}_NODES, currency_ids: &TREE_{}_CURRENCIES }}",
            t.id, t.first_node_id, t.flags, t.id, t.id
        );
        builder.entry(t.id, &value);
    }
    writeln!(out, "pub static TRAIT_TREE_DB: phf::Map<u32, TraitTreeInfo> = {};", builder.build())?;
    writeln!(out)?;
    println!("  TRAIT_TREE_DB: {} trees", trees.len());
    Ok(())
}

pub fn write_node_db(
    out: &mut File,
    nodes: &[NodeInfo],
) -> Result<(), Box<dyn std::error::Error>> {
    for n in nodes {
        emit_u32_array(out, &format!("NODE_{}_ENTRIES", n.id), &n.entry_ids)?;
        emit_u32_array(out, &format!("NODE_{}_CONDS", n.id), &n.cond_ids)?;
        emit_edge_array(out, &format!("NODE_{}_EDGES", n.id), &n.edges)?;
        emit_u32_array(out, &format!("NODE_{}_GROUPS", n.id), &n.group_ids)?;
    }
    let mut builder = phf_codegen::Map::new();
    for n in nodes {
        let value = format!(
            "TraitNodeInfo {{ id: {}, tree_id: {}, pos_x: {}, pos_y: {}, \
             node_type: {}, flags: {}, sub_tree_id: {}, \
             entry_ids: &NODE_{id}_ENTRIES, cond_ids: &NODE_{id}_CONDS, \
             edges: &NODE_{id}_EDGES, group_ids: &NODE_{id}_GROUPS }}",
            n.id, n.tree_id, n.pos_x, n.pos_y, n.node_type, n.flags, n.sub_tree_id,
            id = n.id
        );
        builder.entry(n.id, &value);
    }
    writeln!(out, "pub static TRAIT_NODE_DB: phf::Map<u32, TraitNodeInfo> = {};", builder.build())?;
    writeln!(out)?;
    println!("  TRAIT_NODE_DB: {} nodes", nodes.len());
    Ok(())
}

pub fn write_entry_db(
    out: &mut File,
    entries: &[RawEntry],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = phf_codegen::Map::new();
    for e in entries {
        let value = format!(
            "TraitEntryInfo {{ id: {}, definition_id: {}, max_ranks: {}, \
             entry_type: {}, sub_tree_id: {} }}",
            e.id, e.definition_id, e.max_ranks, e.entry_type, e.sub_tree_id
        );
        builder.entry(e.id, &value);
    }
    writeln!(out, "pub static TRAIT_ENTRY_DB: phf::Map<u32, TraitEntryInfo> = {};", builder.build())?;
    writeln!(out)?;
    println!("  TRAIT_ENTRY_DB: {} entries", entries.len());
    Ok(())
}

pub fn write_definition_db(
    out: &mut File,
    defs: &[RawDefinition],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = phf_codegen::Map::new();
    for d in defs {
        let value = format!(
            "TraitDefInfo {{ id: {}, spell_id: {}, override_icon: {}, \
             overrides_spell_id: {}, visible_spell_id: {}, \
             override_name: \"{}\", override_subtext: \"{}\", \
             override_description: \"{}\" }}",
            d.id, d.spell_id, d.override_icon, d.overrides_spell_id, d.visible_spell_id,
            escape_str(&d.override_name), escape_str(&d.override_subtext),
            escape_str(&d.override_description)
        );
        builder.entry(d.id, &value);
    }
    writeln!(out, "pub static TRAIT_DEFINITION_DB: phf::Map<u32, TraitDefInfo> = {};", builder.build())?;
    writeln!(out)?;
    println!("  TRAIT_DEFINITION_DB: {} definitions", defs.len());
    Ok(())
}

pub fn write_cond_db(
    out: &mut File,
    conds: &[RawCond],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = phf_codegen::Map::new();
    for c in conds {
        let value = format!(
            "TraitCondInfo {{ id: {}, cond_type: {}, tree_id: {}, \
             granted_ranks: {}, quest_id: {}, achievement_id: {}, \
             spec_set_id: {}, group_id: {}, node_id: {}, entry_id: {}, \
             currency_id: {}, spent_amount: {}, flags: {}, required_level: {} }}",
            c.id, c.cond_type, c.tree_id, c.granted_ranks, c.quest_id,
            c.achievement_id, c.spec_set_id, c.group_id, c.node_id,
            c.entry_id, c.currency_id, c.spent_amount, c.flags, c.required_level
        );
        builder.entry(c.id, &value);
    }
    writeln!(out, "pub static TRAIT_COND_DB: phf::Map<u32, TraitCondInfo> = {};", builder.build())?;
    writeln!(out)?;
    println!("  TRAIT_COND_DB: {} conditions", conds.len());
    Ok(())
}

pub fn write_sub_tree_db(
    out: &mut File,
    sub_trees: &[RawSubTree],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = phf_codegen::Map::new();
    for st in sub_trees {
        let value = format!(
            "TraitSubTreeInfo {{ id: {}, name: \"{}\", description: \"{}\", \
             atlas_element_id: {}, tree_id: {} }}",
            st.id, escape_str(&st.name), escape_str(&st.description),
            st.atlas_element_id, st.tree_id
        );
        builder.entry(st.id, &value);
    }
    writeln!(out, "pub static TRAIT_SUBTREE_DB: phf::Map<u32, TraitSubTreeInfo> = {};", builder.build())?;
    writeln!(out)?;
    println!("  TRAIT_SUBTREE_DB: {} subtrees", sub_trees.len());
    Ok(())
}

pub fn write_currency_db(
    out: &mut File,
    currencies: &[RawCurrency],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = phf_codegen::Map::new();
    for c in currencies {
        let value = format!(
            "TraitCurrencyInfo {{ id: {}, currency_type: {}, flags: {} }}",
            c.id, c.currency_type, c.flags
        );
        builder.entry(c.id, &value);
    }
    writeln!(out, "pub static TRAIT_CURRENCY_DB: phf::Map<u32, TraitCurrencyInfo> = {};", builder.build())?;
    writeln!(out)?;
    println!("  TRAIT_CURRENCY_DB: {} currencies", currencies.len());
    Ok(())
}

pub fn write_tests(out: &mut File) -> std::io::Result<()> {
    writeln!(out, "#[cfg(test)]")?;
    writeln!(out, "mod tests {{")?;
    writeln!(out, "    use super::*;")?;
    writeln!(out)?;
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_tree_count() {{")?;
    writeln!(out, "        assert!(TRAIT_TREE_DB.len() > 100);")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_paladin_tree_790() {{")?;
    writeln!(out, "        let tree = TRAIT_TREE_DB.get(&790).expect(\"tree 790\");")?;
    writeln!(out, "        assert_eq!(tree.node_ids.len(), 237);")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_node_has_entries() {{")?;
    writeln!(out, "        let tree = TRAIT_TREE_DB.get(&790).expect(\"tree 790\");")?;
    writeln!(out, "        let first_node_id = tree.node_ids[0];")?;
    writeln!(out, "        let node = TRAIT_NODE_DB.get(&first_node_id).expect(\"first node\");")?;
    writeln!(out, "        assert!(!node.entry_ids.is_empty());")?;
    writeln!(out, "    }}")?;
    writeln!(out)?;
    writeln!(out, "    #[test]")?;
    writeln!(out, "    fn test_definition_count() {{")?;
    writeln!(out, "        assert!(TRAIT_DEFINITION_DB.len() > 10_000);")?;
    writeln!(out, "    }}")?;
    writeln!(out, "}}")?;
    Ok(())
}

// ── Helpers for emitting static arrays ──

fn emit_u32_array(out: &mut File, name: &str, values: &[u32]) -> std::io::Result<()> {
    write!(out, "static {}: [u32; {}] = [", name, values.len())?;
    for (i, v) in values.iter().enumerate() {
        if i > 0 { write!(out, ", ")?; }
        write!(out, "{}", v)?;
    }
    writeln!(out, "];")?;
    Ok(())
}

fn emit_edge_array(
    out: &mut File,
    name: &str,
    edges: &[(u32, u32, u32)],
) -> std::io::Result<()> {
    write!(out, "static {}: [TraitEdgeInfo; {}] = [", name, edges.len())?;
    for (i, (src, etype, vstyle)) in edges.iter().enumerate() {
        if i > 0 { write!(out, ", ")?; }
        write!(out, "TraitEdgeInfo {{ source_node_id: {}, edge_type: {}, visual_style: {} }}", src, etype, vstyle)?;
    }
    writeln!(out, "];")?;
    Ok(())
}
