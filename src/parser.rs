use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufRead, BufReader},
    path::PathBuf,
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TsConfigCompilerOptions {
    pub paths: HashMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct TsConfig {
    #[serde(rename(deserialize = "compilerOptions"))]
    pub compiler_options: TsConfigCompilerOptions,
}

impl TsConfig {
    pub fn default() -> Self {
        let mut paths = HashMap::new();
        paths.insert("@src/*".to_string(), Vec::from(["src/*".to_string()]));
        TsConfig {
            compiler_options: TsConfigCompilerOptions { paths },
        }
    }
}

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub field_type: String,
    pub is_optional: bool,
}

#[derive(Debug)]
pub struct Model {
    pub name: String,
    pub fields: Vec<Field>,
}

pub fn parse_schema(reader: BufReader<File>) -> Vec<Model> {
    let mut lines = reader.lines().peekable();
    let mut models = Vec::new();

    while let Some(Ok(line)) = lines.next() {
        let line = line.trim();

        if line.starts_with("model") {
            let model_name = line.split_whitespace().nth(1).unwrap().to_string();
            let mut fields = Vec::new();

            while let Some(Ok(field_line)) = lines.peek() {
                let field_line = field_line.trim();
                if field_line == "}" {
                    lines.next();
                    break;
                }

                if let Some(field) = parse_field(field_line) {
                    fields.push(field);
                }

                lines.next();
            }

            models.push(Model {
                name: model_name,
                fields,
            });
        }
    }

    models
}

fn parse_field(line: &str) -> Option<Field> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() >= 2 {
        let field_name = parts[0].to_string();
        let mut field_type = parts[1].to_string();
        let is_optional = field_type.ends_with('?');

        if is_optional {
            field_type.pop();
        }

        return Some(Field {
            name: field_name,
            field_type,
            is_optional,
        });
    }

    None
}

pub fn get_schemas(path: String) -> Result<Vec<PathBuf>, io::Error> {
    let entries = fs::read_dir(path)?;

    let file_paths: Vec<_> = entries
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                if e.metadata().map(|m| m.is_file()).unwrap_or(false) {
                    return Some(e.path());
                }

                None
            })
        })
        .collect();

    Ok(file_paths)
}
