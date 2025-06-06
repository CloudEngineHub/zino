#![doc = include_str!("../README.md")]
#![doc(html_favicon_url = "https://zino.cc/assets/zino-logo.png")]
#![doc(html_logo_url = "https://zino.cc/assets/zino-logo.svg")]

use ahash::{HashMap, HashMapExt};
use convert_case::{Case, Casing};
use serde_json::json;
use std::{
    collections::BTreeMap,
    fs::{self, DirEntry},
    io::{self, ErrorKind},
    path::PathBuf,
    sync::OnceLock,
};
use toml::{Table, Value};
use utoipa::openapi::{
    OpenApi, OpenApiBuilder,
    content::ContentBuilder,
    external_docs::ExternalDocs,
    info::{Contact, Info, License},
    path::{PathItem, Paths, PathsBuilder},
    response::ResponseBuilder,
    schema::{
        Components, ComponentsBuilder, KnownFormat, Object, ObjectBuilder, Ref, SchemaFormat, Type,
    },
    security::SecurityRequirement,
    server::Server,
    tag::Tag,
};
use zino_core::{
    LazyLock, Uuid,
    application::{Agent, Application},
    extension::TomlTableExt,
};

mod model;
mod parser;

pub use model::translate_model_entry;

/// Gets the [OpenAPI](https://spec.openapis.org/oas/latest.html) document.
pub fn openapi() -> OpenApi {
    OpenApiBuilder::new()
        .paths(default_paths()) // should come first to load OpenAPI files
        .components(Some(default_components()))
        .tags(Some(default_tags()))
        .servers(Some(default_servers()))
        .security(Some(default_securities()))
        .external_docs(default_external_docs())
        .info(openapi_info(Agent::name(), Agent::version()))
        .build()
}

/// Constructs the OpenAPI `Info` object.
fn openapi_info(title: &str, version: &str) -> Info {
    let mut info = Info::new(title, version);
    if let Some(config) = OPENAPI_INFO.get() {
        if let Some(title) = config.get_str("title") {
            title.clone_into(&mut info.title);
        }
        if let Some(description) = config.get_str("description") {
            info.description = Some(description.to_owned());
        }
        if let Some(terms_of_service) = config.get_str("terms_of_service") {
            info.terms_of_service = Some(terms_of_service.to_owned());
        }
        if let Some(contact_config) = config.get_table("contact") {
            let mut contact = Contact::new();
            if let Some(contact_name) = contact_config.get_str("name") {
                contact.name = Some(contact_name.to_owned());
            }
            if let Some(contact_url) = contact_config.get_str("url") {
                contact.url = Some(contact_url.to_owned());
            }
            if let Some(contact_email) = contact_config.get_str("email") {
                contact.email = Some(contact_email.to_owned());
            }
            info.contact = Some(contact);
        }
        if let Some(license) = config.get_str("license") {
            info.license = Some(License::new(license));
        } else if let Some(license_config) = config.get_table("license") {
            let license_name = license_config.get_str("name").unwrap_or_default();
            let mut license = License::new(license_name);
            if let Some(license_url) = license_config.get_str("url") {
                license.url = Some(license_url.to_owned());
            }
            info.license = Some(license);
        }
        if let Some(version) = config.get_str("version") {
            version.clone_into(&mut info.version);
        }
    }
    info
}

/// Returns the default OpenAPI paths.
fn default_paths() -> Paths {
    let mut paths_builder = PathsBuilder::new();
    for (path, item) in OPENAPI_PATHS.iter() {
        paths_builder = paths_builder.path(path, item.to_owned());
    }
    paths_builder.build()
}

/// Returns the default OpenAPI components.
fn default_components() -> Components {
    let mut components = OPENAPI_COMPONENTS.get_or_init(Components::new).to_owned();

    // Request ID
    let request_id_example = Uuid::now_v7();
    let request_id_schema = ObjectBuilder::new()
        .schema_type(Type::String)
        .format(Some(SchemaFormat::KnownFormat(KnownFormat::Uuid)))
        .build();

    // Default response
    let status_schema = ObjectBuilder::new()
        .schema_type(Type::Integer)
        .examples(Some(200))
        .build();
    let success_schema = ObjectBuilder::new()
        .schema_type(Type::Boolean)
        .examples(Some(true))
        .build();
    let message_schema = ObjectBuilder::new()
        .schema_type(Type::String)
        .examples(Some("OK"))
        .build();
    let default_response_schema = ObjectBuilder::new()
        .schema_type(Type::Object)
        .property("status", status_schema)
        .property("success", success_schema)
        .property("message", message_schema)
        .property("request_id", request_id_schema.clone())
        .property("data", Object::new())
        .required("status")
        .required("success")
        .required("message")
        .required("request_id")
        .build();
    let default_response_example = json!({
        "status": 200,
        "success": true,
        "message": "OK",
        "request_id": request_id_example,
        "data": {},
    });
    let default_response_content = ContentBuilder::new()
        .schema(Some(Ref::from_schema_name("defaultResponse")))
        .example(Some(default_response_example))
        .build();
    let default_response = ResponseBuilder::new()
        .content("application/json", default_response_content)
        .build();
    components
        .schemas
        .insert("defaultResponse".to_owned(), default_response_schema.into());
    components
        .responses
        .insert("default".to_owned(), default_response.into());

    // 4XX error response
    let model_id_example = Uuid::now_v7();
    let detail_example = format!("404 Not Found: cannot find the model `{model_id_example}`");
    let instance_example = format!("/model/{model_id_example}/view");
    let status_schema = ObjectBuilder::new()
        .schema_type(Type::Integer)
        .examples(Some(404))
        .build();
    let success_schema = ObjectBuilder::new()
        .schema_type(Type::Boolean)
        .examples(Some(false))
        .build();
    let title_schema = ObjectBuilder::new()
        .schema_type(Type::String)
        .examples(Some("NotFound"))
        .build();
    let detail_schema = ObjectBuilder::new()
        .schema_type(Type::String)
        .examples(Some(detail_example.as_str()))
        .build();
    let instance_schema = ObjectBuilder::new()
        .schema_type(Type::String)
        .examples(Some(instance_example.as_str()))
        .build();
    let error_response_schema = ObjectBuilder::new()
        .schema_type(Type::Object)
        .property("status", status_schema)
        .property("success", success_schema)
        .property("title", title_schema)
        .property("detail", detail_schema)
        .property("instance", instance_schema)
        .property("request_id", request_id_schema)
        .required("status")
        .required("success")
        .required("title")
        .required("detail")
        .required("instance")
        .required("request_id")
        .build();
    let error_response_example = json!({
        "status": 404,
        "success": false,
        "title": "NotFound",
        "detail": detail_example,
        "instance": instance_example,
        "request_id": request_id_example,
    });
    let error_response_content = ContentBuilder::new()
        .schema(Some(Ref::from_schema_name("errorResponse")))
        .example(Some(error_response_example))
        .build();
    let error_response = ResponseBuilder::new()
        .content("application/json", error_response_content)
        .build();
    components
        .schemas
        .insert("errorResponse".to_owned(), error_response_schema.into());
    components
        .responses
        .insert("4XX".to_owned(), error_response.into());

    components
}

/// Returns the default OpenAPI tags.
fn default_tags() -> Vec<Tag> {
    OPENAPI_TAGS.get_or_init(Vec::new).to_owned()
}

/// Returns the default OpenAPI servers.
fn default_servers() -> Vec<Server> {
    OPENAPI_SERVERS
        .get_or_init(|| vec![Server::new("/")])
        .to_owned()
}

/// Returns the default OpenAPI security requirements.
fn default_securities() -> Vec<SecurityRequirement> {
    OPENAPI_SECURITIES.get_or_init(Vec::new).to_owned()
}

/// Returns the default OpenAPI external docs.
fn default_external_docs() -> Option<ExternalDocs> {
    OPENAPI_EXTERNAL_DOCS.get().cloned()
}

/// Parses the OpenAPI directory.
fn parse_openapi_dir(
    dir: PathBuf,
    tags: &mut Vec<Tag>,
    paths: &mut BTreeMap<String, PathItem>,
    definitions: &mut HashMap<&str, Table>,
    builder: Option<ComponentsBuilder>,
) -> Result<ComponentsBuilder, io::Error> {
    let mut builder = builder.unwrap_or_default();
    let mut dirs = Vec::new();
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            dirs.push(path);
        } else if path.is_file() {
            files.push(entry);
        }
    }
    for file in files {
        if file.file_name() == "OPENAPI.toml" {
            builder = parse_openapi_metadata(file, builder);
        } else {
            let data = parse_openapi_model(file, paths, definitions, builder);
            builder = data.0;
            tags.push(data.1);
        }
    }
    for dir in dirs {
        builder = parse_openapi_dir(dir, tags, paths, definitions, Some(builder))?;
    }
    Ok(builder)
}

/// Parses the OpenAPI metadata file.
fn parse_openapi_metadata(file: DirEntry, mut builder: ComponentsBuilder) -> ComponentsBuilder {
    let path = file.path();
    let mut config = fs::read_to_string(&path)
        .unwrap_or_else(|err| {
            let path = path.display();
            panic!("fail to read the OpenAPI metadata file `{path}`: {err}");
        })
        .parse::<Table>()
        .expect("fail to parse the OpenAPI metadata file as a TOML table");
    if let Some(Value::Table(info)) = config.remove("info") {
        if OPENAPI_INFO.set(info).is_err() {
            panic!("fail to set OpenAPI info");
        }
    }
    if let Some(servers) = config.get_array("servers") {
        let servers = servers
            .iter()
            .filter_map(|v| v.as_table())
            .map(parser::parse_server)
            .collect::<Vec<_>>();
        if OPENAPI_SERVERS.set(servers).is_err() {
            panic!("fail to set OpenAPI servers");
        }
    }
    if let Some(security_schemes) = config.get_table("security_schemes") {
        for (name, scheme) in security_schemes {
            if let Some(scheme_config) = scheme.as_table() {
                let scheme = parser::parse_security_scheme(scheme_config);
                builder = builder.security_scheme(name, scheme);
            }
        }
    }
    if let Some(securities) = config.get_array("securities") {
        let security_requirements = securities
            .iter()
            .filter_map(|v| v.as_table())
            .map(parser::parse_security_requirement)
            .collect::<Vec<_>>();
        if OPENAPI_SECURITIES.set(security_requirements).is_err() {
            panic!("fail to set OpenAPI security requirements");
        }
    }
    if let Some(external_docs) = config.get_table("external_docs") {
        let external_docs = parser::parse_external_docs(external_docs);
        if OPENAPI_EXTERNAL_DOCS.set(external_docs).is_err() {
            panic!("fail to set OpenAPI external docs");
        }
    }
    builder
}

/// Parses the OpenAPI model file.
fn parse_openapi_model(
    file: DirEntry,
    paths: &mut BTreeMap<String, PathItem>,
    definitions: &mut HashMap<&str, Table>,
    mut builder: ComponentsBuilder,
) -> (ComponentsBuilder, Tag) {
    let path = file.path();
    let config = fs::read_to_string(&path)
        .unwrap_or_else(|err| {
            let path = path.display();
            panic!("fail to read the OpenAPI model file `{path}`: {err}");
        })
        .parse::<Table>()
        .expect("fail to parse the OpenAPI model file as a TOML table");
    let tag_name = config
        .get_str("name")
        .map(|s| s.to_owned())
        .unwrap_or_else(|| {
            file.file_name()
                .to_string_lossy()
                .trim_end_matches(".toml")
                .to_owned()
        });
    let ignore_securities = config.get_array("securities").is_some_and(|v| v.is_empty());
    if let Some(endpoints) = config.get_array("endpoints") {
        for endpoint in endpoints.iter().filter_map(|v| v.as_table()) {
            let path = endpoint.get_str("path").unwrap_or("/");
            let method = endpoint
                .get_str("method")
                .unwrap_or_default()
                .to_ascii_uppercase();
            let http_method = parser::parse_http_method(&method);
            let operation = parser::parse_operation(&tag_name, path, endpoint, ignore_securities);
            let path_item = PathItem::new(http_method, operation);
            if let Some(item) = paths.get_mut(path) {
                item.merge_operations(path_item);
            } else {
                paths.insert(path.to_owned(), path_item);
            }
        }
    }
    if let Some(schemas) = config.get_table("schemas") {
        for (key, value) in schemas.iter() {
            if let Some(config) = value.as_table() {
                let name = key.to_case(Case::Camel);
                let schema = parser::parse_schema(config);
                builder = builder.schema(name, schema);
            }
        }
    }
    if let Some(responses) = config.get_table("responses") {
        for (key, value) in responses.iter() {
            if let Some(config) = value.as_table() {
                let name = key.to_case(Case::Camel);
                let response = parser::parse_response(config);
                builder = builder.response(name, response);
            }
        }
    }
    if let Some(models) = config.get_table("models") {
        for (model_name, model_fields) in models {
            if let Some(fields) = model_fields.as_table() {
                let name = model_name.to_owned().leak() as &'static str;
                definitions.insert(name, fields.to_owned());
            }
        }
    }
    (builder, parser::parse_tag(&tag_name, &config))
}

/// OpenAPI paths.
static OPENAPI_PATHS: LazyLock<BTreeMap<String, PathItem>> = LazyLock::new(|| {
    let mut tags = Vec::new();
    let mut paths = BTreeMap::new();
    let mut definitions = HashMap::new();
    let openapi_dir = Agent::config_dir().join("openapi");
    match parse_openapi_dir(openapi_dir, &mut tags, &mut paths, &mut definitions, None) {
        Ok(components_builder) => {
            if OPENAPI_COMPONENTS.set(components_builder.build()).is_err() {
                panic!("fail to set OpenAPI components");
            }
            if OPENAPI_TAGS.set(tags).is_err() {
                panic!("fail to set OpenAPI tags");
            }
            if MODEL_DEFINITIONS.set(definitions).is_err() {
                panic!("fail to set model definitions");
            }
        }
        Err(err) => {
            if err.kind() != ErrorKind::NotFound {
                tracing::error!("{err}");
            }
        }
    }
    paths
});

/// OpenAPI info.
static OPENAPI_INFO: OnceLock<Table> = OnceLock::new();

/// OpenAPI components.
static OPENAPI_COMPONENTS: OnceLock<Components> = OnceLock::new();

/// OpenAPI tags.
static OPENAPI_TAGS: OnceLock<Vec<Tag>> = OnceLock::new();

/// OpenAPI servers.
static OPENAPI_SERVERS: OnceLock<Vec<Server>> = OnceLock::new();

/// OpenAPI securities.
static OPENAPI_SECURITIES: OnceLock<Vec<SecurityRequirement>> = OnceLock::new();

/// OpenAPI external docs.
static OPENAPI_EXTERNAL_DOCS: OnceLock<ExternalDocs> = OnceLock::new();

/// Model definitions.
static MODEL_DEFINITIONS: OnceLock<HashMap<&str, Table>> = OnceLock::new();
