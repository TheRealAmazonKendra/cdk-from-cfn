use noctilucent::ir::CloudformationProgramIr;
use noctilucent::synthesizer::typescript_synthesizer::TypescriptSynthesizer;
use noctilucent::CloudformationParseTree;

macro_rules! test_case {
    ($name:ident) => {
        #[test]
        fn $name() {
            let expected = include_str!(concat!("end-to-end/", stringify!($name), "/app.ts"));

            let actual = {
                let mut output = Vec::with_capacity(expected.len());

                let cfn: CloudformationParseTree = serde_yaml::from_str(include_str!(concat!(
                    "end-to-end/",
                    stringify!($name),
                    "/template.yml"
                )))
                .unwrap();
                let ir = CloudformationProgramIr::from(cfn).unwrap();
                ir.synthesize(&TypescriptSynthesizer {}, &mut output)
                    .unwrap();
                String::from_utf8(output).unwrap()
            };

            let _update_snapshots = UpdateSnapshot::new(
                concat!("end-to-end/", stringify!($name), "/app.ts"),
                &actual,
            );

            assert_eq!(expected, actual);
        }
    };
}

test_case!(simple);
test_case!(vpc);

struct UpdateSnapshot<'a> {
    path: &'static str,
    actual: &'a str,
}

impl<'a> UpdateSnapshot<'a> {
    fn new(path: &'static str, actual: &'a str) -> Self {
        Self { path, actual }
    }
}

impl Drop for UpdateSnapshot<'_> {
    fn drop(&mut self) {
        use std::fs::File;
        use std::io::Write;
        use std::path::PathBuf;

        if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join(self.path);
            let mut file = File::create(path).unwrap();
            file.write_all(self.actual.as_bytes()).unwrap();
        }
    }
}