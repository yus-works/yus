use once_cell::sync::Lazy;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LangMeta {
    pub color: Option<String>,
    pub devicon: Option<String>,
}

const YML: &str = include_str!("../data/languages.yml");

pub static LANG_TABLE: Lazy<HashMap<String, LangMeta>> = Lazy::new(|| {
    let docs = yaml_rust::YamlLoader::load_from_str(YML).expect("parse languages.yml");
    let yml = docs[0].clone();

    static DEVICON: &[(&str, &str)] = &[
        ("Rust", "rust-plain"),
        ("Go", "go-original"),
        ("JavaScript", "javascript-plain"),
        ("TypeScript", "typescript-plain"),
        ("HTML", "html5-plain"),
        ("CSS", "css3-plain"),
        ("Python", "python-plain"),
        ("C", "c-plain"),
        ("C++", "cplusplus-plain"),
        ("GLSL", "opengl-plain"),
    ];

    let devicon_map: HashMap<&str, &str> = DEVICON.iter().cloned().collect();

    devicon_map.into_iter()
        .map(|(name, icon)| {
            (
                name.to_owned(),
                LangMeta {
                    color: Some(yml[name]["color"].as_str().unwrap().to_owned()),
                    devicon: Some(icon.to_owned()),
                },
            )
        })
        .collect::<HashMap<String, LangMeta>>()
});
