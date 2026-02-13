//! CSV loaders for trait data.

use super::csv_util::{parse_csv_line, wow_data_dir};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

// ── Raw CSV structs ──

pub struct RawTree {
    pub id: u32,
    pub first_node_id: u32,
    pub flags: u32,
}

pub struct RawNode {
    pub id: u32,
    pub tree_id: u32,
    pub pos_x: i32,
    pub pos_y: i32,
    pub node_type: u32,
    pub flags: u32,
    pub sub_tree_id: u32,
}

pub struct RawEntry {
    pub id: u32,
    pub definition_id: u32,
    pub max_ranks: u32,
    pub entry_type: u32,
    pub sub_tree_id: u32,
}

pub struct RawDefinition {
    pub id: u32,
    pub spell_id: u32,
    pub override_icon: u32,
    pub overrides_spell_id: u32,
    pub visible_spell_id: u32,
    pub override_name: String,
    pub override_subtext: String,
    pub override_description: String,
}

pub struct RawEdge {
    #[allow(dead_code)]
    pub id: u32,
    pub visual_style: u32,
    pub left_node_id: u32,
    pub right_node_id: u32,
    pub edge_type: u32,
}

pub struct RawCond {
    pub id: u32,
    pub cond_type: u32,
    pub tree_id: u32,
    pub granted_ranks: u32,
    pub quest_id: u32,
    pub achievement_id: u32,
    pub spec_set_id: u32,
    pub group_id: u32,
    pub node_id: u32,
    pub entry_id: u32,
    pub currency_id: u32,
    pub spent_amount: u32,
    pub flags: u32,
    pub required_level: u32,
}

pub struct RawSubTree {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub atlas_element_id: u32,
    pub tree_id: u32,
}

pub struct RawCurrency {
    pub id: u32,
    pub currency_type: u32,
    pub flags: u32,
}

// ── Join table maps ──

pub struct JoinMaps {
    pub node_to_entries: HashMap<u32, Vec<u32>>,
    pub node_to_conds: HashMap<u32, Vec<u32>>,
    pub tree_to_currencies: HashMap<u32, Vec<u32>>,
    pub node_to_groups: HashMap<u32, Vec<u32>>,
    pub group_to_conds: HashMap<u32, Vec<u32>>,
}

/// Load all raw data from CSV files.
pub struct AllData {
    pub trees: Vec<RawTree>,
    pub nodes: Vec<RawNode>,
    pub entries: Vec<RawEntry>,
    pub definitions: Vec<RawDefinition>,
    pub edges: Vec<RawEdge>,
    pub conds: Vec<RawCond>,
    pub sub_trees: Vec<RawSubTree>,
    pub currencies: Vec<RawCurrency>,
    pub joins: JoinMaps,
}

pub fn load_all() -> Result<AllData, Box<dyn std::error::Error>> {
    let data_dir = wow_data_dir();

    let trees = load_trees(&data_dir)?;
    let nodes = load_nodes(&data_dir)?;
    let entries = load_entries(&data_dir)?;
    let definitions = load_definitions(&data_dir)?;
    let edges = load_edges(&data_dir)?;
    let conds = load_conds(&data_dir)?;
    let sub_trees = load_sub_trees(&data_dir)?;
    let currencies = load_currencies(&data_dir)?;

    println!("Trees: {}, Nodes: {}, Entries: {}, Definitions: {}",
        trees.len(), nodes.len(), entries.len(), definitions.len());
    println!("Edges: {}, Conds: {}, SubTrees: {}, Currencies: {}",
        edges.len(), conds.len(), sub_trees.len(), currencies.len());

    let joins = load_join_tables(&data_dir)?;
    Ok(AllData { trees, nodes, entries, definitions, edges, conds, sub_trees, currencies, joins })
}

fn load_join_tables(data_dir: &Path) -> Result<JoinMaps, Box<dyn std::error::Error>> {
    let node_to_entries = load_pair_csv(
        &data_dir.join("TraitNodeXTraitNodeEntry.csv"), 1, 2,
    )?;
    let node_to_conds = load_pair_csv(
        &data_dir.join("TraitNodeXTraitCond.csv"), 2, 1,
    )?;
    let tree_to_currencies = load_pair_csv(
        &data_dir.join("TraitTreeXTraitCurrency.csv"), 2, 3,
    )?;
    let node_to_groups = load_pair_csv(
        &data_dir.join("TraitNodeGroupXTraitNode.csv"), 2, 1,
    )?;
    // TraitNodeGroupXTraitCond: ID, TraitCondID, TraitNodeGroupID
    let group_to_conds = load_pair_csv(
        &data_dir.join("TraitNodeGroupXTraitCond.csv"), 2, 1,
    )?;
    Ok(JoinMaps { node_to_entries, node_to_conds, tree_to_currencies, node_to_groups, group_to_conds })
}

/// Load a CSV join table, collecting values from `val_col` grouped by `key_col`.
fn load_pair_csv(
    path: &Path,
    key_col: usize,
    val_col: usize,
) -> Result<HashMap<u32, Vec<u32>>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut map: HashMap<u32, Vec<u32>> = HashMap::new();

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if i == 0 { continue; }
        let fields = parse_csv_line(&line);
        if fields.len() <= key_col.max(val_col) { continue; }
        let key: u32 = match fields[key_col].parse() { Ok(v) => v, Err(_) => continue };
        let val: u32 = match fields[val_col].parse() { Ok(v) => v, Err(_) => continue };
        map.entry(key).or_default().push(val);
    }
    Ok(map)
}

// ── Individual CSV loaders ──

fn load_trees(data_dir: &Path) -> Result<Vec<RawTree>, Box<dyn std::error::Error>> {
    let file = File::open(data_dir.join("TraitTree.csv"))?;
    let reader = BufReader::new(file);
    let mut trees = Vec::new();
    // TitleText_lang,ID,TraitSystemID,TraitTreeID,FirstTraitNodeID,PlayerConditionID,Flags,...
    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if i == 0 { continue; }
        let f = parse_csv_line(&line);
        if f.len() < 7 { continue; }
        let id: u32 = match f[1].parse() { Ok(v) => v, Err(_) => continue };
        trees.push(RawTree {
            id,
            first_node_id: f[4].parse().unwrap_or(0),
            flags: f[6].parse().unwrap_or(0),
        });
    }
    Ok(trees)
}

fn load_nodes(data_dir: &Path) -> Result<Vec<RawNode>, Box<dyn std::error::Error>> {
    let file = File::open(data_dir.join("TraitNode.csv"))?;
    let reader = BufReader::new(file);
    let mut nodes = Vec::new();
    // ID,TraitTreeID,PosX,PosY,Type,Flags,TraitSubTreeID
    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if i == 0 { continue; }
        let f = parse_csv_line(&line);
        if f.len() < 7 { continue; }
        nodes.push(RawNode {
            id: f[0].parse()?,
            tree_id: f[1].parse()?,
            pos_x: f[2].parse().unwrap_or(0),
            pos_y: f[3].parse().unwrap_or(0),
            node_type: f[4].parse().unwrap_or(0),
            flags: f[5].parse().unwrap_or(0),
            sub_tree_id: f[6].parse().unwrap_or(0),
        });
    }
    Ok(nodes)
}

fn load_entries(data_dir: &Path) -> Result<Vec<RawEntry>, Box<dyn std::error::Error>> {
    let file = File::open(data_dir.join("TraitNodeEntry.csv"))?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    // ID,TraitDefinitionID,MaxRanks,NodeEntryType,TraitSubTreeID
    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if i == 0 { continue; }
        let f = parse_csv_line(&line);
        if f.len() < 5 { continue; }
        entries.push(RawEntry {
            id: f[0].parse()?,
            definition_id: f[1].parse().unwrap_or(0),
            max_ranks: f[2].parse().unwrap_or(1),
            entry_type: f[3].parse().unwrap_or(0),
            sub_tree_id: f[4].parse().unwrap_or(0),
        });
    }
    Ok(entries)
}

fn load_definitions(data_dir: &Path) -> Result<Vec<RawDefinition>, Box<dyn std::error::Error>> {
    let file = File::open(data_dir.join("TraitDefinition.csv"))?;
    let reader = BufReader::new(file);
    let mut defs = Vec::new();
    // OverrideName_lang,OverrideSubtext_lang,OverrideDescription_lang,ID,SpellID,OverrideIcon,OverridesSpellID,VisibleSpellID
    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if i == 0 { continue; }
        let f = parse_csv_line(&line);
        if f.len() < 8 { continue; }
        defs.push(RawDefinition {
            override_name: f[0].clone(),
            override_subtext: f[1].clone(),
            override_description: f[2].clone(),
            id: f[3].parse()?,
            spell_id: f[4].parse().unwrap_or(0),
            override_icon: f[5].parse().unwrap_or(0),
            overrides_spell_id: f[6].parse().unwrap_or(0),
            visible_spell_id: f[7].parse().unwrap_or(0),
        });
    }
    Ok(defs)
}

fn load_edges(data_dir: &Path) -> Result<Vec<RawEdge>, Box<dyn std::error::Error>> {
    let file = File::open(data_dir.join("TraitEdge.csv"))?;
    let reader = BufReader::new(file);
    let mut edges = Vec::new();
    // ID,VisualStyle,LeftTraitNodeID,RightTraitNodeID,Type
    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if i == 0 { continue; }
        let f = parse_csv_line(&line);
        if f.len() < 5 { continue; }
        edges.push(RawEdge {
            id: f[0].parse()?,
            visual_style: f[1].parse().unwrap_or(0),
            left_node_id: f[2].parse().unwrap_or(0),
            right_node_id: f[3].parse().unwrap_or(0),
            edge_type: f[4].parse().unwrap_or(0),
        });
    }
    Ok(edges)
}

fn load_conds(data_dir: &Path) -> Result<Vec<RawCond>, Box<dyn std::error::Error>> {
    let file = File::open(data_dir.join("TraitCond.csv"))?;
    let reader = BufReader::new(file);
    let mut conds = Vec::new();
    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if i == 0 { continue; }
        let f = parse_csv_line(&line);
        if f.len() < 14 { continue; }
        conds.push(RawCond {
            id: f[0].parse()?,
            cond_type: f[1].parse().unwrap_or(0),
            tree_id: f[2].parse().unwrap_or(0),
            granted_ranks: f[3].parse().unwrap_or(0),
            quest_id: f[4].parse().unwrap_or(0),
            achievement_id: f[5].parse().unwrap_or(0),
            spec_set_id: f[6].parse().unwrap_or(0),
            group_id: f[7].parse().unwrap_or(0),
            node_id: f[8].parse().unwrap_or(0),
            entry_id: f[9].parse().unwrap_or(0),
            currency_id: f[10].parse().unwrap_or(0),
            spent_amount: f[11].parse().unwrap_or(0),
            flags: f[12].parse().unwrap_or(0),
            required_level: f[13].parse().unwrap_or(0),
        });
    }
    Ok(conds)
}

fn load_sub_trees(data_dir: &Path) -> Result<Vec<RawSubTree>, Box<dyn std::error::Error>> {
    let file = File::open(data_dir.join("TraitSubTree.csv"))?;
    let reader = BufReader::new(file);
    let mut trees = Vec::new();
    // Name_lang,Description_lang,ID,UiTextureAtlasElementID,TraitTreeID
    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if i == 0 { continue; }
        let f = parse_csv_line(&line);
        if f.len() < 5 { continue; }
        trees.push(RawSubTree {
            name: f[0].clone(),
            description: f[1].clone(),
            id: f[2].parse()?,
            atlas_element_id: f[3].parse().unwrap_or(0),
            tree_id: f[4].parse().unwrap_or(0),
        });
    }
    Ok(trees)
}

fn load_currencies(data_dir: &Path) -> Result<Vec<RawCurrency>, Box<dyn std::error::Error>> {
    let file = File::open(data_dir.join("TraitCurrency.csv"))?;
    let reader = BufReader::new(file);
    let mut currencies = Vec::new();
    // ID,Type,CurrencyTypesID,Flags,...
    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if i == 0 { continue; }
        let f = parse_csv_line(&line);
        if f.len() < 4 { continue; }
        currencies.push(RawCurrency {
            id: f[0].parse()?,
            currency_type: f[1].parse().unwrap_or(0),
            flags: f[3].parse().unwrap_or(0),
        });
    }
    Ok(currencies)
}
