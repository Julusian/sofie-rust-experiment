use std::{collections::HashMap, hash::Hash};

use nanoid::nanoid;

/**
 * Limited characterset to use for id generation
 * Generating id's using these characters has 2 reasons:
 * 1. By omitting 0, O, I, 1 it makes it easier to read for humans
 * 2. The Timeline only supports A-Za-z0-9 in id's and classnames
 */
const UNMISTAKABLE_CHARS: [char; 55] = [
    '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L',
    'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
    'i', 'j', 'k', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];

pub fn get_random_id() -> String {
    nanoid!(17, &UNMISTAKABLE_CHARS)
}

/**
 * Convert an array to a Map, keyed on an id generator function.
 * `undefined` key values will get filtered from the map
 * Duplicate keys will cause entries to replace others silently
 *
 * ```
 * normalizeArrayToMapFunc([{ a: 1, b: 2}], (o) => o.a + o.b)
 * ```
 */
pub fn normalizeArrayToMapOfRefs<'a, T, K: Eq + Hash, F: Fn(&T) -> Option<K>>(
    array: &'a [T],
    get_key: F,
) -> HashMap<K, &'a T> {
    let mut result = HashMap::new();

    for item in array {
        if let Some(key) = get_key(item) {
            result.insert(key, item);
        }
    }

    result
}
