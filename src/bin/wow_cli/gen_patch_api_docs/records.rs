use super::Result;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(PartialEq)]
pub(super) struct Record {
    pub category: &'static str,
    pub payload: Value,
}
pub(super) type Records = BTreeMap<String, Record>;

fn name(value: &Value, key: &str) -> Result<String> {
    value[key]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| format!("missing string {key} in declaration: {value}").into())
}

fn list<'a>(value: &'a Value, key: &str) -> Result<&'a [Value]> {
    match value.get(key) {
        None => Ok(&[]),
        Some(Value::Array(values)) => Ok(values),
        _ => Err(format!("unsupported {key} list shape: {value}").into()),
    }
}

fn qualify(namespace: &str, name: &str) -> String {
    if namespace.is_empty() {
        name.to_owned()
    } else {
        format!("{namespace}.{name}")
    }
}

fn insert(
    records: &mut Records,
    symbol: String,
    category: &'static str,
    payload: Value,
) -> Result<()> {
    if records.contains_key(&symbol) {
        return Err(format!("duplicate identity: {symbol}").into());
    }
    records.insert(symbol, Record { category, payload });
    Ok(())
}

fn without(value: &Value, keys: &[&str]) -> Value {
    let mut value = value.clone();
    if let Some(object) = value.as_object_mut() {
        for key in keys {
            object.remove(*key);
        }
    }
    value
}

pub(super) fn insert_document(records: &mut Records, document: Value) -> Result<()> {
    if !document.is_object() {
        return Err("unsupported documentation root".into());
    }
    let namespace = document["Namespace"].as_str().unwrap_or("");
    let object = match document.get("Type") {
        None => None,
        Some(Value::String(kind)) if kind == "System" => None,
        Some(Value::String(kind)) if kind == "ScriptObject" => {
            let raw = name(&document, "Name")?;
            let object = raw.strip_suffix("API").unwrap_or(&raw).to_owned();
            let mut parent = without(&document, &["Functions", "Events", "Tables", "Predicates"]);
            parent["Name"] = Value::String(object.clone());
            insert(records, object.clone(), "script-object", parent)?;
            Some(object)
        }
        kind => return Err(format!("unsupported documentation declaration type: {kind:?}").into()),
    };
    insert_functions(records, &document, namespace, object.as_deref())?;
    for event in list(&document, "Events")? {
        require_type(event, &["Event"])?;
        insert(records, name(event, "LiteralName")?, "event", event.clone())?;
    }
    for table in list(&document, "Tables")? {
        insert_table(records, table, namespace)?;
    }
    for predicate in list(&document, "Predicates")? {
        require_type(predicate, &["Secret", "Precondition", "Predicate"])?;
        insert(
            records,
            qualify(namespace, &name(predicate, "Name")?),
            "predicate",
            predicate.clone(),
        )?;
    }
    Ok(())
}

fn require_type(value: &Value, allowed: &[&str]) -> Result<()> {
    let kind = name(value, "Type")?;
    if !allowed.contains(&kind.as_str()) {
        return Err(format!("unsupported declaration type {kind}").into());
    }
    Ok(())
}

fn insert_functions(
    records: &mut Records,
    document: &Value,
    namespace: &str,
    object: Option<&str>,
) -> Result<()> {
    for function in list(document, "Functions")? {
        require_type(function, &["Function"])?;
        let prefix = function["Namespace"]
            .as_str()
            .unwrap_or(object.unwrap_or(namespace));
        let symbol = qualify(prefix, &name(function, "Name")?);
        let category = if object.is_some() {
            "method"
        } else {
            "function"
        };
        insert(records, symbol, category, without(function, &["Namespace"]))?;
    }
    Ok(())
}

fn insert_table(records: &mut Records, table: &Value, namespace: &str) -> Result<()> {
    let kind = name(table, "Type")?;
    let table_name = name(table, "Name")?;
    let (symbol, category, children, child_category) = match kind.as_str() {
        "Structure" => (
            qualify(namespace, &table_name),
            "structure",
            "Fields",
            "field",
        ),
        "Enumeration" => (
            format!("Enum.{table_name}"),
            "enum",
            "Fields",
            "enum-member",
        ),
        "Constants" => (
            format!("Constants.{table_name}"),
            "constants",
            "Values",
            "constant",
        ),
        "CallbackType" => {
            return insert(
                records,
                qualify(namespace, &table_name),
                "callback",
                table.clone(),
            );
        }
        _ => return Err(format!("unsupported table declaration type: {kind}").into()),
    };
    // Parent records retain metadata and ordered fields, alongside member records.
    insert(records, symbol.clone(), category, table.clone())?;
    for field in list(table, children)? {
        insert(
            records,
            qualify(&symbol, &name(field, "Name")?),
            child_category,
            field.clone(),
        )?;
    }
    Ok(())
}
