use serde::Deserialize;
use std::collections::HashMap;

#[test]
fn struct_() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Test {
        a: u8,
        b: u16,
        c: u32,
        d: u64,
        e: i8,
        f: i16,
        g: i32,
        h: i64,
        i: f32,
        j: f64,
        k: String,
        l: Option<String>,
        m: (),
    }

    let cni = r#"
        a=255
        b=65535
        c=4294967295
        d=18446744073709551615
        e=-128
        f=-32768
        g=-2147483648
        h=-9223372036854775808
        i=-3.4028235e38
        j=-1.7976931348623157e308
        k=All human beings are born free and equal in dignity and rights.
        l=``
        m=
	"#;

    assert_eq!(
        Ok(Test {
            a: 255,
            b: 65_535,
            c: 4_294_967_295,
            d: 18_446_744_073_709_551_615,
            e: -128,
            f: -32768,
            g: -2147483648,
            h: -9223372036854775808,
            i: -3.4028235e38_f32,
            j: -1.7976931348623157e308_f64,
            k: "All human beings are born free and equal in dignity and rights.".into(),
            l: None,
            m: (),
        }),
        crate::from_str::<Test>(cni)
    );
}

#[test]
fn map() {
    let cni = r#"
        a=255
        b=65535
        c=4294967295
        d=18446744073709551615
        e=-128
        f=-32768
        g=-2147483648
        h=-9223372036854775808
        i=-3.4028235e38
        j=-1.7976931348623157e308
        k=All human beings are born free and equal in dignity and rights.
        l=``
        m=
	"#;

    let mut map: HashMap<String, String> = HashMap::new();
    map.insert("a".into(), "255".into());
    map.insert("b".into(), "65535".into());
    map.insert("c".into(), "4294967295".into());
    map.insert("d".into(), "18446744073709551615".into());
    map.insert("e".into(), "-128".into());
    map.insert("f".into(), "-32768".into());
    map.insert("g".into(), "-2147483648".into());
    map.insert("h".into(), "-9223372036854775808".into());
    map.insert("i".into(), "-3.4028235e38".into());
    map.insert("j".into(), "-1.7976931348623157e308".into());
    map.insert(
        "k".into(),
        "All human beings are born free and equal in dignity and rights.".into(),
    );
    map.insert("l".into(), "".into());
    map.insert("m".into(), "".into());

    assert_eq!(Ok(map), crate::from_str::<HashMap<String, String>>(cni));
}
