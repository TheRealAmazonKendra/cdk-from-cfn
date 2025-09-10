struct LanguageData {
    pub name: &'static str,
    pub app_file: &'static str,
    pub lang_name: &'static str,
    pub stack_path: &'static str,
}

pub struct Language;

impl Language {
    pub const TYPESCRIPT: &'static str = "typescript";
    pub const PYTHON: &'static str = "python";
    pub const JAVA: &'static str = "java";
    pub const GOLANG: &'static str = "golang";
    pub const CSHARP: &'static str = "csharp";

    const CONFIGS: &'static [LanguageData] = &[
        LanguageData {
            name: Self::TYPESCRIPT,
            app_file: "app.ts",
            lang_name: Self::TYPESCRIPT,
            stack_path: "{}.ts",
        },
        LanguageData {
            name: Self::PYTHON,
            app_file: "app.py",
            lang_name: Self::PYTHON,
            stack_path: "{}.py",
        },
        LanguageData {
            name: Self::JAVA,
            app_file: "src/main/java/com/myorg/MyApp.java",
            lang_name: Self::JAVA,
            stack_path: "src/main/java/com/myorg/{}.java",
        },
        LanguageData {
            name: Self::GOLANG,
            app_file: "app.go",
            lang_name: "go",
            stack_path: "{}.go",
        },
        LanguageData {
            name: Self::CSHARP,
            app_file: "Program.cs",
            lang_name: Self::CSHARP,
            stack_path: "{}.cs",
        },
    ];

    fn get_property<T, F>(lang: &str, property_fn: F) -> T
    where
        F: FnOnce(&LanguageData) -> T,
    {
        Self::CONFIGS
            .iter()
            .find(|config| config.name == lang)
            .map(property_fn)
            .unwrap()
    }

    pub fn lang_arg(lang: &str) -> &str {
        Self::get_property(lang, |config| config.lang_name)
    }

    pub fn app_name(lang: &str) -> &'static str {
        Self::get_property(lang, |config| config.app_file)
    }

    pub fn post_process_output(lang: &str, mut content: String) -> String {
        if lang == Self::GOLANG {
            if let Some(main) = content.find("func main()") {
                content.truncate(main);
            }
        }
        content
    }

    pub fn all_languages() -> Vec<String> {
        Self::CONFIGS
            .iter()
            .map(|config| config.name.to_string())
            .collect()
    }

    pub fn stack_filename(lang: &str, stack_name: &str) -> String {
        let stack_path = Self::get_property(lang, |config| config.stack_path);
        stack_path.replace("{}", stack_name)
    }
}