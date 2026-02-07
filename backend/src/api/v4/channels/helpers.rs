pub(super) fn normalize_notify_props(value: serde_json::Value) -> serde_json::Value {
    if value.is_null() {
        return serde_json::json!({"desktop": "default", "mark_unread": "all"});
    }

    if let Some(obj) = value.as_object() {
        if obj.is_empty() {
            return serde_json::json!({"desktop": "default", "mark_unread": "all"});
        }
    }

    value
}

#[cfg(test)]
mod tests {
    use super::normalize_notify_props;

    #[test]
    fn normalizes_null_notify_props() {
        let result = normalize_notify_props(serde_json::Value::Null);
        assert_eq!(
            result,
            serde_json::json!({"desktop": "default", "mark_unread": "all"})
        );
    }

    #[test]
    fn keeps_existing_notify_props() {
        let input = serde_json::json!({"desktop": "all"});
        let result = normalize_notify_props(input.clone());
        assert_eq!(result, input);
    }
}
