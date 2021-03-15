//! Tests for the formatter. Only the macro is changed so the test definitions
//! can be reused from src/tests/mod.rs

macro_rules! cni_test (
	($name:ident, $path:expr) => {
		#[test]
		fn $name(){
			let data = serde_json::from_str(include_str!(concat!($path, ".json"))).unwrap();
			assert_eq!(
				crate::from_str(&crate::to_str(&data)).unwrap(),
				data
			);
		}
	};
	($name:ident, $path:expr, fail) => {
		// do nothing as there is no json file
		// this is just here so the rest of the file can be copied vebatim
		// from src/tests/mod.rs
	};
);

mod core {
    cni_test!(bareword01, "cni/tests/core/bareword/01");
    cni_test!(bareword02, "cni/tests/core/bareword/02");
    cni_test!(bareword04, "cni/tests/core/bareword/04", fail);

    cni_test!(comment01, "cni/tests/core/comment/01");
    cni_test!(comment02, "cni/tests/core/comment/02");
    cni_test!(comment03, "cni/tests/core/comment/03");
    cni_test!(comment04, "cni/tests/core/comment/04");
    cni_test!(comment05, "cni/tests/core/comment/05", fail);

    cni_test!(key01, "cni/tests/core/key/01");
    cni_test!(key02, "cni/tests/core/key/02");
    cni_test!(key03, "cni/tests/core/key/03");
    cni_test!(key04, "cni/tests/core/key/04", fail);
    cni_test!(key05, "cni/tests/core/key/05", fail);
    cni_test!(key06, "cni/tests/core/key/06", fail);
    cni_test!(key09, "cni/tests/core/key/09", fail);

    cni_test!(raw01, "cni/tests/core/raw/01");
    cni_test!(raw02, "cni/tests/core/raw/02");
    cni_test!(raw03, "cni/tests/core/raw/03");
    cni_test!(raw04, "cni/tests/core/raw/04", fail);

    cni_test!(section01, "cni/tests/core/section/01");
    cni_test!(section02, "cni/tests/core/section/02");
    cni_test!(section03, "cni/tests/core/section/03");
    cni_test!(section04, "cni/tests/core/section/04", fail);
    cni_test!(section05, "cni/tests/core/section/05", fail);
    cni_test!(section06, "cni/tests/core/section/06", fail);
    cni_test!(section07, "cni/tests/core/section/07", fail);
    cni_test!(section08, "cni/tests/core/section/08", fail);
    cni_test!(section09, "cni/tests/core/section/09", fail);

    cni_test!(flexspace, "cni/tests/core/flexspace");
    cni_test!(sect_and_key, "cni/tests/core/sect_and_key");
    cni_test!(unicode, "cni/tests/core/unicode", fail);
}

mod ini {
    cni_test!(ini01, "cni/tests/ini/01");
}

mod ext {
    #[cfg(feature = "more-keys")]
    cni_test!(more_keys, "cni/tests/ext/more-keys");
}

mod bundle {
    cni_test!(exotic, "cni/tests/bundle/exotic");
    cni_test!(common, "cni/tests/bundle/common");
}
