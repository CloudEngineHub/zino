/// Generates SQL `SET` expressions.
use super::{query::QueryExt, DatabaseDriver, Schema};
use crate::model::{EncodeColumn, Mutation, Query};

/// Extension trait for [`Mutation`](crate::model::Mutation).
pub(super) trait MutationExt<DB> {
    /// Formats the updates to generate SQL `SET` expression.
    fn format_updates<M: Schema>(&self) -> String;
}

impl MutationExt<DatabaseDriver> for Mutation {
    fn format_updates<M: Schema>(&self) -> String {
        let updates = self.updates();
        if updates.is_empty() {
            return String::new();
        }

        let fields = self.fields();
        let permissive = fields.is_empty();
        let readonly_fields = M::readonly_fields();
        let mut mutations = Vec::new();
        for (key, value) in updates.iter() {
            if (permissive || fields.contains(key))
                && !readonly_fields.contains(&key.as_str())
                && let Some(col) = M::get_column(key)
            {
                let key = Query::format_field(key);
                let Some(map) = value.as_object() else {
                    let value = col.encode_value(Some(value));
                    let mutation = format!(r#"{key} = {value}"#);
                    mutations.push(mutation);
                    continue;
                };

                let mut set_json_object = true;
                for (operator, value) in map {
                    match operator.as_str() {
                        "$inc" => {
                            let value = col.encode_value(Some(value));
                            let mutation = format!(r#"{key} = {key} + {value}"#);
                            mutations.push(mutation);
                        }
                        "$mul" => {
                            let value = col.encode_value(Some(value));
                            let mutation = format!(r#"{key} = {key} * {value}"#);
                            mutations.push(mutation);
                        }
                        "$add" => {
                            if let Some(values) = value.as_array() && values.len() >= 2 {
                                let value = values.iter()
                                    .map(|v| {
                                        if let Some(s) = v.as_str() && M::has_column(s) {
                                            Query::format_field(s)
                                        } else {
                                            col.encode_value(Some(v))
                                        }
                                    })
                                    .collect::<Vec<_>>()
                                    .join(" + ");
                                let mutation = format!(r#"{key} = {value}"#);
                                mutations.push(mutation);
                            }
                        }
                        "$multiply" => {
                            if let Some(values) = value.as_array() && values.len() >= 2 {
                                let value = values.iter()
                                    .map(|v| {
                                        if let Some(s) = v.as_str() && M::has_column(s) {
                                            Query::format_field(s)
                                        } else {
                                            col.encode_value(Some(v))
                                        }
                                    })
                                    .collect::<Vec<_>>()
                                    .join(" * ");
                                let mutation = format!(r#"{key} = {value}"#);
                                mutations.push(mutation);
                            }
                        }
                        "$subtract" => {
                            if let Some(values) = value.as_array() && values.len() == 2 {
                                let value = values.iter()
                                    .map(|v| {
                                        if let Some(s) = v.as_str() && M::has_column(s) {
                                            Query::format_field(s)
                                        } else {
                                            col.encode_value(Some(v))
                                        }
                                    })
                                    .collect::<Vec<_>>()
                                    .join(" - ");
                                let mutation = format!(r#"{key} = {value}"#);
                                mutations.push(mutation);
                            }
                        }
                        "$divide" => {
                            if let Some(values) = value.as_array() && values.len() == 2 {
                                let value = values.iter()
                                    .map(|v| {
                                        if let Some(s) = v.as_str() && M::has_column(s) {
                                            Query::format_field(s)
                                        } else {
                                            col.encode_value(Some(v))
                                        }
                                    })
                                    .collect::<Vec<_>>()
                                    .join(" / ");
                                let mutation = format!(r#"{key} = {value}"#);
                                mutations.push(mutation);
                            }
                        }
                        "$min" => {
                            if let Some(values) = value.as_array() && values.len() >= 2 {
                                let value = values.iter()
                                    .map(|v| {
                                        if let Some(s) = v.as_str() && M::has_column(s) {
                                            Query::format_field(s)
                                        } else {
                                            col.encode_value(Some(v))
                                        }
                                    })
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                let mutation = format!(r#"{key} = LEAST({value})"#);
                                mutations.push(mutation);
                            }
                        }
                        "$max" => {
                            if let Some(values) = value.as_array() && values.len() >= 2 {
                                let value = values.iter()
                                    .map(|v| {
                                        if let Some(s) = v.as_str() && M::has_column(s) {
                                            Query::format_field(s)
                                        } else {
                                            col.encode_value(Some(v))
                                        }
                                    })
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                let mutation = format!(r#"{key} = GREATEST({value})"#);
                                mutations.push(mutation);
                            }
                        }
                        _ => ()
                    }
                    if operator.starts_with('$') {
                        set_json_object = false;
                        break;
                    }
                }
                if set_json_object {
                    let value = col.encode_value(Some(&value));
                    let mutation = format!(r#"{key} = {value}"#);
                    mutations.push(mutation);
                }
            }
        }
        mutations.join(", ")
    }
}
