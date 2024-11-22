use crate::parser::{Field, Model};
use core::fmt;
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use std::{fs, path::Path};

const ENTITY_PATH: &str = "domain/entity/";
const MAPPER_PATH: &str = "infra/database/prisma/mappers";
const REPOSITORY_PATH: &str = "app/repositories";
const PRISMA_REPOSITORY_PATH: &str = "infra/database/prisma";

#[derive(Debug, PartialEq, Eq)]
pub enum ModuleType {
    Entity,
    Mapper,
    Repository(Option<Vec<RepositoryOperations>>),
    PrismaRepository,
}

impl From<&str> for ModuleType {
    fn from(value: &str) -> Self {
        match value {
            "Entity" => ModuleType::Entity,
            "Mapper" => ModuleType::Mapper,
            "Repository" => ModuleType::Repository(None),
            "Prisma repository" => ModuleType::PrismaRepository,
            _ => unreachable!(),
        }
    }
}

impl From<ModuleType> for &str {
    fn from(value: ModuleType) -> Self {
        match value {
            ModuleType::Entity => "Entity",
            ModuleType::Mapper => "Mapper",
            ModuleType::Repository(_) => "Repository",
            ModuleType::PrismaRepository => "Prisma repository",
        }
    }
}

fn lowercase_first_char(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(first) => first.to_lowercase().collect::<String>() + c.as_str(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryOperations {
    Create,
    Find,
    FindMany,
    Delete,
    Update,
}

impl fmt::Display for RepositoryOperations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            RepositoryOperations::Create => write!(f, "create"),
            RepositoryOperations::Find => write!(f, "find"),
            RepositoryOperations::FindMany => write!(f, "findMany"),
            RepositoryOperations::Delete => write!(f, "delete"),
            RepositoryOperations::Update => write!(f, "update"),
        }
    }
}

fn build_repository_methods(
    model_name: &str,
    input_type: &str,
    return_type: &str,
    has_mapper: bool,
    op: &RepositoryOperations,
) -> String {
    match op {
        RepositoryOperations::Create => {
            let mut method = format!(
                "async create(data: {}): Promise<{}> {{\n",
                input_type, return_type
            );
            if has_mapper {
                write!(
                    method,
                    r#"    const result = await this.prisma.{}.create({{
      data,
    }})

    return {}Mapper.toDomain(result)
  }}"#,
                    lowercase_first_char(model_name),
                    model_name
                )
                .unwrap();

                return method;
            }

            write!(
                method,
                r#"      return this.prisma.{}.create({{
        data,
      }})
  }}"#,
                lowercase_first_char(model_name)
            )
            .unwrap();

            method
        }
        RepositoryOperations::Delete => format!(
            r#"async delete(id: string) {{
    await this.prisma.{}.update({{
      where: {{
        id,
      }},
      data: {{
        deletedAt: new Date(),
      }},
    }})
  }}"#,
            lowercase_first_char(model_name)
        ),
        RepositoryOperations::Find => {
            let mut method = format!(
                "async find(data: {}): Promise<{}> {{\n",
                input_type, return_type
            );

            if has_mapper {
                write!(
                    method,
                    r#"    const result = await this.prisma.{}.findFirst({{
      where: data,
    }})

    return {}Mapper.toDomain(result)
  }}"#,
                    lowercase_first_char(model_name),
                    model_name
                )
                .unwrap();

                return method;
            }

            write!(
                method,
                r#"      return this.prisma.{}.findFirst({{
        where: data,
      }})
  }}"#,
                lowercase_first_char(model_name)
            )
            .unwrap();

            method
        }
        RepositoryOperations::FindMany => {
            let mut method = format!(
                "async findMany(data: {}): Promise<{}[]> {{\n",
                input_type, return_type
            );

            if has_mapper {
                write!(
                    method,
                    r#"    const result = await this.prisma.{}.findMany({{
      where: data,
    }})

    return result.map({}Mapper.toDomain)
  }}"#,
                    lowercase_first_char(model_name),
                    model_name
                )
                .unwrap();

                return method;
            }

            write!(
                method,
                r#"      return this.prisma.{}.findMany({{
        where: data,
      }})
  }}"#,
                lowercase_first_char(model_name)
            )
            .unwrap();

            method
        }
        RepositoryOperations::Update => {
            let mut method = format!(
                "async update(id: string, data: {}): Promise<{}> {{\n",
                input_type, return_type
            );

            if has_mapper {
                write!(
                    method,
                    r#"    const result = await this.prisma.{}.update({{
      where: {{
        id,
      }},
      data,
    }})

    return {}Mapper.toDomain(result)
  }}"#,
                    lowercase_first_char(model_name),
                    model_name
                )
                .unwrap();

                return method;
            }

            write!(
                method,
                r#"      return this.prisma.{}.findMany({{
        where: data,
      }})
  }}"#,
                lowercase_first_char(model_name)
            )
            .unwrap();

            method
        }
    }
}

fn create_repository(
    model: &Model,
    methods: Option<Vec<RepositoryOperations>>,
    has_mapper: bool,
    has_entity: bool,
) -> (String, String) {
    let mut abstract_repository = format!("export abstract class {}Repository {{", model.name);
    let mut prisma_repository = format!(
        r#"@Injectable()
export class Prisma{}Repository implements {}Repository {{
    constructor(private readonly prisma: PrismaService) {{}}"#,
        model.name, model.name
    );

    let (input_type, return_type) = if has_entity {
        (format!("Partial<{}>", model.name), model.name.clone())
    } else {
        ("any".to_string(), "any".to_string())
    };

    let methods = methods.unwrap_or_default();

    for method in &methods {
        match method {
            RepositoryOperations::Create => write!(
                abstract_repository,
                "\n    abstract create(data: {}): Promise<{}>",
                input_type, return_type
            )
            .unwrap(),
            RepositoryOperations::Find => write!(
                abstract_repository,
                "\n    abstract find(data: {}): Promise<{}>",
                input_type, return_type
            )
            .unwrap(),
            RepositoryOperations::FindMany => write!(
                abstract_repository,
                "\n    abstract findMany(data: {}): Promise<{}[]>",
                input_type, return_type
            )
            .unwrap(),
            RepositoryOperations::Update => write!(
                abstract_repository,
                "\n    abstract update(id: string, data: {}): Promise<{}>",
                input_type, return_type
            )
            .unwrap(),
            RepositoryOperations::Delete => write!(
                abstract_repository,
                "\n    abstract delete(id: string): Promise<void>"
            )
            .unwrap(),
        }

        write!(
            prisma_repository,
            "\n    {}",
            build_repository_methods(&model.name, &input_type, &return_type, has_mapper, method)
        )
        .unwrap();
    }

    write!(abstract_repository, "\n}}").unwrap();
    write!(prisma_repository, "\n}}").unwrap();

    (abstract_repository, prisma_repository)
}

fn create_mapper(model: &Model) -> String {
    let mut mapper = String::new();
    write!(
        mapper,
        "export class {}Mapper {{\n  static toDomain(data: Prisma{}): {} {{\n    return new {}({{",
        model.name, model.name, model.name, model.name
    )
    .unwrap();

    for field in &model.fields {
        if get_field_with_type(field, false).is_some() {
            match field.field_type.as_str() {
                "Decimal" | "BigInt" => write!(
                    mapper,
                    "\n      {}: Number(data.{}),",
                    field.name, field.name
                )
                .unwrap(),
                _ => write!(mapper, "\n      {}: data.{},", field.name, field.name).unwrap(),
            }
        }
    }

    write!(mapper, "\n    }})\n  }}\n}}").unwrap();

    mapper
}

fn create_entity(model: &Model) -> String {
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
        "\n\n  constructor({}: {}) {{\n    Object.assign(this, {})\n  }}\n}}",
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
            "\n  readonly {}: {}",
            field_name, field_type
        )
        .unwrap();
    } else {
        write!(formatted_field_type, "\n  {}: {}", field_name, field_type).unwrap();
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

fn to_kebab_case(name: &str) -> String {
    let mut kebab_case_string = String::new();
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            kebab_case_string.push('-');
        }
        kebab_case_string.push(ch.to_ascii_lowercase());
    }

    kebab_case_string
}

fn build_path(dir: &Path, module_path: &str, module_type: ModuleType, model_name: &str) -> String {
    let kebab_model_name = to_kebab_case(model_name);
    let (path, file_name) = match module_type {
        ModuleType::Entity => (ENTITY_PATH, format!("{}.entity.ts", kebab_model_name)),
        ModuleType::Mapper => (MAPPER_PATH, format!("{}.mapper.ts", kebab_model_name)),
        ModuleType::Repository(_) => (
            REPOSITORY_PATH,
            format!("{}.repository.ts", kebab_model_name),
        ),
        ModuleType::PrismaRepository => (
            PRISMA_REPOSITORY_PATH,
            format!("prisma-{}.repository.ts", kebab_model_name),
        ),
    };

    format!("{}/{}{}/{}", dir.display(), module_path, path, file_name)
}

fn write_to_module<P: AsRef<Path>>(path: P, contents: String) -> std::io::Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = fs::File::create(path)?;
    file.write_all(contents.as_bytes())?;

    Ok(())
}

pub fn write_modules(modules: Vec<ModuleType>, dir: &Path, module_path: &str, model: &Model) {
    for module in &modules {
        match module {
            ModuleType::Entity => write_to_module(
                build_path(dir, module_path, ModuleType::Entity, &model.name),
                create_entity(model),
            )
            .unwrap(),
            ModuleType::Mapper => write_to_module(
                build_path(dir, module_path, ModuleType::Mapper, &model.name),
                create_mapper(model),
            )
            .unwrap(),
            ModuleType::Repository(methods) => {
                let (abstract_repository, prisma_repository) = create_repository(
                    model,
                    methods.clone(),
                    modules.contains(&ModuleType::Mapper),
                    modules.contains(&ModuleType::Entity),
                );

                write_to_module(
                    build_path(dir, module_path, ModuleType::Repository(None), &model.name),
                    abstract_repository,
                )
                .unwrap();

                write_to_module(
                    build_path(dir, module_path, ModuleType::PrismaRepository, &model.name),
                    prisma_repository,
                )
                .unwrap();
            }
            _ => unreachable!(),
        }
    }
}
