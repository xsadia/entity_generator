use crate::parser::{Field, Model};
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use std::{fs, path::Path};

const MAPPER_PATH: &str = "domain/entity/";
const ENTITY_PATH: &str = "infra/database/prisma/mappers";

pub enum ModuleType {
    Entity,
    Mapper,
}

fn lowercase_first_char(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(first) => first.to_lowercase().collect::<String>() + c.as_str(),
    }
}

pub fn create_mapper(model: &Model) -> String {
    let mut mapper = String::new();
    write!(
        mapper,
        "export class {}Mapper {{\n\tstatic toDomain(data: Prisma{}): {} {{\n\t\treturn new {}({{",
        model.name, model.name, model.name, model.name
    )
    .unwrap();

    for field in &model.fields {
        if get_field_with_type(field, false).is_some() {
            write!(mapper, "\n\t\t\t{}: data.{},", field.name, field.name).unwrap();
        }
    }

    write!(mapper, "\n\t\t}})\n\t}}\n}}").unwrap();

    mapper
}

pub fn create_entity(model: &Model) -> String {
    let entity_interface = String::from("I") + &model.name;
    let mut entity = String::new();

    write!(entity, "export interface {} {{", entity_interface).unwrap();

    for field in &model.fields {
        let parsed_field_option = get_field_with_type(field, false);

        if let Some(parsed_field) = parsed_field_option {
            entity.push_str(&parsed_field);
        }
    }

    entity.push_str("\n}\n\n");

    write!(
        entity,
        "export class {} implements {} {{",
        model.name, entity_interface
    )
    .unwrap();

    for field in &model.fields {
        let parsed_field_option = get_field_with_type(field, true);
        if let Some(parsed_field) = parsed_field_option {
            entity.push_str(&parsed_field);
        }
    }

    let param_name = lowercase_first_char(&model.name);

    writeln!(
        entity,
        "\n\n\tconstructor({}: {}) {{\n\t\tObject.assign(this, {})\n\t}}\n}}",
        param_name, entity_interface, param_name,
    )
    .unwrap();

    entity
}

fn build_type_string(
    field_type: &str,
    field_name: &str,
    is_optional: bool,
    read_only: bool,
) -> String {
    let mut formatted_field_type = String::new();
    if read_only {
        write!(
            formatted_field_type,
            "\n\treadonly {}: {}",
            field_name, field_type
        )
        .unwrap();
    } else {
        write!(formatted_field_type, "\n\t{}: {}", field_name, field_type).unwrap();
    };

    if is_optional {
        write!(formatted_field_type, " | null").unwrap();
    }

    formatted_field_type
}

fn get_field_with_type(field: &Field, read_only: bool) -> Option<String> {
    match field.field_type.as_str() {
        "Float" | "Int" | "Decimal" | "BigInt" => Some(build_type_string(
            "number",
            &field.name,
            field.is_optional,
            read_only,
        )),
        "String" => Some(build_type_string(
            "string",
            &field.name,
            field.is_optional,
            read_only,
        )),
        "Boolean" => Some(build_type_string(
            "boolean",
            &field.name,
            field.is_optional,
            read_only,
        )),
        "DateTime" => Some(build_type_string(
            "Date",
            &field.name,
            field.is_optional,
            read_only,
        )),
        _ => None,
    }
}

pub fn build_path(
    dir: &Path,
    module_path: &str,
    module_type: ModuleType,
    model_name: &str,
) -> String {
    let (path, file_extension) = match module_type {
        ModuleType::Entity => (ENTITY_PATH, ".entity.ts"),
        ModuleType::Mapper => (MAPPER_PATH, ".mapper.ts"),
    };
    format!(
        "{}/{}{}/{}{}",
        dir.display(),
        module_path,
        path,
        model_name,
        file_extension
    )
}

pub fn write_to_module<P: AsRef<Path>>(path: P, contents: String) -> std::io::Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = fs::File::create(path)?;
    file.write_all(contents.as_bytes())?;

    Ok(())
}
