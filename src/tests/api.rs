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
    use crate::api::Cni;

    let mut map = HashMap::<String, String>::new();
    map.insert("b".into(), "b".into());
    map.insert("c".into(), "c".into());
    map.insert("b.c".into(), "c".into());

    assert_eq!(test_map().sub_tree("a"), map);
}

#[test]
fn sub_leaves() {
    use crate::api::Cni;

    let mut map = HashMap::<String, String>::new();
    map.insert("b".into(), "b".into());
    map.insert("c".into(), "c".into());

    assert_eq!(test_map().sub_leaves("a"), map);
}

#[test]
fn walk_tree() {
    use crate::api::CniIter;

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
}

#[test]
fn walk_leaves() {
    use crate::api::CniIter;

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
}
