use serde_json::Value;

pub(crate) mod endpoints;
pub mod search;

/// Recurses over an object and returns the first string it finds, or `None` if it never
/// finds anything.
pub(crate) fn crawl_object_for_string<'a>(object: &'a Value, ignore: &[&str]) -> Option<&'a str> {
    // I know this is odd.
    if let Some(object) = object.as_object() {
        for (name, x) in object {
            // Doesn't currently contain anything we want, but does have text.
            if ignore.contains(&name.as_str()) {
                continue;
            }

            if let Some(s) = x.as_str() {
                return Some(s);
            } else if let Some(x) = x.as_array() {
                for x in x {
                    if let Some(x) = crawl_object_for_string(x, ignore) {
                        return Some(x);
                    }
                }
            } else if x.is_object() {
                if let Some(x) = crawl_object_for_string(x, ignore) {
                    return Some(x);
                }
            } else {
                return None;
            }
        }
        None
    } else {
        None
    }
}
