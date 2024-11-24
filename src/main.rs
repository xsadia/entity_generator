use code_gen::{write_modules, ModuleType, RepositoryOperations};
use dialoguer::{theme::ColorfulTheme, FuzzySelect, MultiSelect};
use parser::{get_schemas, parse_schema, TsConfig};
use std::{
    env,
    fs::{self, File},
    io::BufReader,
};

mod code_gen;
mod parser;

fn main() {
    let dir = env::current_dir().unwrap();

    let schemas = get_schemas(format!("{}/prisma", dir.display()))
        .unwrap_or_else(|_| panic!("prisma schema not found at path {}/prisma", dir.display()));

    let schema_file_names: Vec<String> = schemas
        .iter()
        .filter_map(|schema| {
            schema
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
        })
        .collect();

    let schema_selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select prisma schema")
        .default(0)
        .items(&schema_file_names)
        .interact()
        .unwrap();

    let schema_file = File::open(schemas.get(schema_selection).unwrap()).unwrap();

    let reader = BufReader::new(schema_file);

    let models = parse_schema(reader);

    let model_names: Vec<&str> = models.iter().map(|model| model.name.as_str()).collect();

    let model_selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select model")
        .default(0)
        .items(&model_names)
        .interact()
        .unwrap();

    let selected_model = models.get(model_selection).unwrap();

    let ts_config_content = fs::read_to_string(format!("{}/tsconfig.json", dir.display())).unwrap();

    let ts_config: TsConfig =
        serde_json::from_str(&ts_config_content).unwrap_or_else(|_| TsConfig::default());

    let modules: Vec<String> = ts_config
        .compiler_options
        .paths
        .keys()
        .map(|key| key.replace('@', "").replace("/*", ""))
        .collect();

    let module_selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select output module")
        .default(0)
        .items(&modules)
        .interact()
        .unwrap();

    let selected_module = modules.get(module_selection).unwrap();

    let module_path = ts_config
        .compiler_options
        .paths
        .get(&format!("@{}/*", selected_module))
        .unwrap()
        .first()
        .unwrap()
        .replace("*", "");

    let multiselected: &[&str; 3] = &[
        ModuleType::Entity.into(),
        ModuleType::Mapper.into(),
        ModuleType::Repository(None).into(),
    ];

    let defaults = &[true, false, false];

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select which classes to create")
        .items(&multiselected[..])
        .defaults(&defaults[..])
        .interact()
        .unwrap();

    let mut selected_modules: Vec<ModuleType> = selections
        .iter()
        .map(|i| ModuleType::from(*multiselected.get(*i).unwrap()))
        .collect();

    if selected_modules.contains(&ModuleType::Repository(None)) {
        let methods: &[RepositoryOperations; 5] = &[
            RepositoryOperations::Find,
            RepositoryOperations::FindMany,
            RepositoryOperations::Create,
            RepositoryOperations::Delete,
            RepositoryOperations::Update,
        ];

        let selections = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select which repository methods to create")
            .items(&methods[..])
            .defaults(&defaults[..])
            .interact()
            .unwrap();

        let selected_repositories: Vec<RepositoryOperations> = selections
            .iter()
            .map(|&i| methods.get(i).unwrap().clone())
            .collect();

        let index = selected_modules
            .iter()
            .position(|item| *item == ModuleType::Repository(None))
            .unwrap();

        selected_modules[index] = ModuleType::Repository(Some(selected_repositories))
    };

    write_modules(selected_modules, &dir, &module_path, selected_model)
}
