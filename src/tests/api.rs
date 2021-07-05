use crate::CniExt;
use std::collections::HashMap;

fn test_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("a".into(), "a".into());
    map.insert("a.b".into(), "b".into());
    map.insert("a.c".into(), "c".into());
    map.insert("a.b.c".into(), "c".into());
    map
}

#[test]
fn sub_tree() {
    let mut map = HashMap::<String, String>::new();
    map.insert("b".into(), "b".into());
    map.insert("c".into(), "c".into());
    map.insert("b.c".into(), "c".into());

    assert_eq!(test_map().sub_tree("a"), map);

    assert_eq!(test_map().sub_tree(""), test_map());
}

#[test]
fn sub_leaves() {
    let mut map = HashMap::<String, String>::new();
    map.insert("b".into(), "b".into());
    map.insert("c".into(), "c".into());

    assert_eq!(test_map().sub_leaves("a"), map);

    map.clear();
    map.insert("a".into(), "a".into());

    assert_eq!(test_map().sub_leaves(""), map);
}

#[test]
fn walk_tree() {
    let mut map = HashMap::<String, String>::new();
    map.insert("a.b".into(), "b".into());
    map.insert("a.c".into(), "c".into());
    map.insert("a.b.c".into(), "c".into());

    assert_eq!(
        test_map()
            .iter()
            .walk_tree("a")
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<HashMap<String, String>>(),
        map
    );
    assert_eq!(
        test_map()
            .walk_tree("a")
            .collect::<HashMap<String, String>>(),
        map
    );

    map.insert("a".into(), "a".into());

    assert_eq!(
        test_map()
            .walk_tree("")
            .collect::<HashMap<String, String>>(),
        map
    );
}

#[test]
fn walk_leaves() {
    let mut map = HashMap::<String, String>::new();
    map.insert("a.b".into(), "b".into());
    map.insert("a.c".into(), "c".into());

    assert_eq!(
        test_map()
            .iter()
            .walk_leaves("a")
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<HashMap<String, String>>(),
        map
    );
    assert_eq!(
        test_map()
            .walk_leaves("a")
            .collect::<HashMap<String, String>>(),
        map
    );

    map.clear();
    map.insert("a".into(), "a".into());

    assert_eq!(
        test_map()
            .walk_leaves("")
            .collect::<HashMap<String, String>>(),
        map
    );
}

#[test]
fn section_tree() {
    assert_eq!(
        test_map().section_tree("a").into_iter().collect::<Vec<_>>(),
        vec!["b"]
    );
    assert_eq!(
        test_map().section_tree("").into_iter().collect::<Vec<_>>(),
        vec!["a", "a.b"]
    );
}

#[test]
fn section_leaves() {
    assert_eq!(
        test_map()
            .section_leaves("")
            .into_iter()
            .collect::<Vec<_>>(),
        vec!["a"]
    );
    println!("---");
    assert_eq!(
        test_map()
            .section_leaves("a")
            .into_iter()
            .collect::<Vec<_>>(),
        vec!["b"]
    );
}
