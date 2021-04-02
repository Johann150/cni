//! Tests for the parser. Uses the tests from the git submodule going to the
//! cni specification repository.

macro_rules! cni_test (
	($name:ident, $path:expr) => {
		#[test]
		fn $name(){
			assert_eq!(
				crate::from_str(include_str!(concat!($path, ".cni"))).unwrap(),
				serde_json::from_str(include_str!(concat!($path, ".json"))).unwrap()
			);
		}
	};
	($name:ident, $path:expr, fail) => {
		#[test]
		fn $name(){
			assert!(crate::from_str(include_str!(concat!($path, "_fail.cni"))).is_err());
		}
	};
);

mod api;
mod serializer;

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
    cni_test!(section09, "cni/tests/core/section/09", fail);

    cni_test!(flexspace, "cni/tests/core/flexspace");
    cni_test!(sect_and_key, "cni/tests/core/sect_and_key");
    cni_test!(unicode, "cni/tests/core/unicode");
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
