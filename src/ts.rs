use crate::{js_ts, *};
use heck::{ToLowerCamelCase, ToShoutyKebabCase};
use indoc::formatdoc;
use specta::{
    functions::FunctionDataType,
    js_doc,
    reference::{self, reference},
    ts::{self, datatype, ExportError},
    DataType, DataTypeReference, NamedType, TypeMap,
};
use tauri::Runtime;

/// Implements [`ExportLanguage`] for TypeScript exporting
pub struct Language;

pub fn builder<TRuntime: Runtime>() -> PluginBuilder<Language, NoCommands<TRuntime>, NoEvents> {
    PluginBuilder::default()
}

pub const GLOBALS: &str = include_str!("./globals.ts");

type Config = specta::ts::ExportConfig;
pub type ExportConfig = crate::ExportConfig<Config>;

impl Language {
    fn named_to_reference_type(
        name: String,
        typ: &DataType,
        type_map: &TypeMap,
    ) -> Option<DataTypeReference> {
        let entry = type_map.iter().find(|(_, ndt)| *ndt.name() == name)?;

        if let Some(g) = typ.generics() {
            if !g.is_empty() {
                return None;
            }
        }

        let refer = specta::internal::construct::data_type_reference(name.into(), entry.0, vec![]);

        Some(refer)
    }

    fn wrap_result_type(
        cfg: &Config,
        datatype: &specta::DataType,
        type_map: &TypeMap,
    ) -> Result<String, ExportError> {
        match datatype {
            DataType::Any => ts::datatype(cfg, datatype, type_map),
            DataType::Unknown => ts::datatype(cfg, datatype, type_map),
            DataType::Primitive(_) => ts::datatype(cfg, datatype, type_map),
            DataType::Literal(_) => ts::datatype(cfg, datatype, type_map),
            DataType::List(_) => ts::datatype(cfg, datatype, type_map),
            DataType::Nullable(_) => ts::datatype(cfg, datatype, type_map),
            DataType::Map(_) => ts::datatype(cfg, datatype, type_map),
            DataType::Struct(st) => {
                let name = st.name().to_string();

                let refer = Self::named_to_reference_type(name, datatype, type_map);

                if let Some(refer) = refer {
                    ts::datatype(cfg, &DataType::Reference(refer), type_map)
                } else {
                    ts::datatype(cfg, datatype, type_map)
                }
            }
            DataType::Enum(en) => {
                let name = en.name().to_string();

                let refer = Self::named_to_reference_type(name, datatype, type_map);

                if let Some(refer) = refer {
                    ts::datatype(cfg, &DataType::Reference(refer), type_map)
                } else {
                    ts::datatype(cfg, datatype, type_map)
                }
            }
            DataType::Tuple(_) => ts::datatype(cfg, datatype, type_map),
            DataType::Result(_) => ts::datatype(cfg, datatype, type_map),
            DataType::Reference(_) => ts::datatype(cfg, datatype, type_map),
            DataType::Generic(_) => ts::datatype(cfg, datatype, type_map),
        }
    }
}

impl ExportLanguage for Language {
    type Config = Config;
    type Error = ts::ExportError;

    fn run_format(path: PathBuf, cfg: &ExportConfig) {
        cfg.inner.run_format(path).ok();
    }

    /// Renders a collection of [`FunctionDataType`] into a TypeScript string.
    fn render_commands(
        commands: &[FunctionDataType],
        type_map: &TypeMap,
        cfg: &ExportConfig,
    ) -> Result<String, ExportError> {
        let commands = commands
            .iter()
            .map(|function| {
                let arg_defs = function
                    .args
                    .iter()
                    .map(|(name, typ)| {
                        ts::datatype(&cfg.inner, typ, type_map)
                            .map(|ty| format!("{}: {}", name.to_lower_camel_case(), ty))
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                let arg_type = format!("[{}]", arg_defs.join(", "));

                let ret_type = match &function.result {
                    specta::DataType::Result(t) => {
                        let (r, _) = t.as_ref();

                        Self::wrap_result_type(&cfg.inner, r, type_map)
                    }
                    specta::DataType::Nullable(n) => {
                        Self::wrap_result_type(&cfg.inner, n, type_map)
                    }
                    t => Self::wrap_result_type(&cfg.inner, t, type_map),
                }?;

                let err_type = match &function.result {
                    specta::DataType::Result(t) => {
                        let (_, e) = t.as_ref();

                        Self::wrap_result_type(&cfg.inner, e, type_map)
                    }
                    _ => Ok("never".to_string()),
                }?;

                let name = function.name.to_string();

                let full_key = cfg
                    .plugin_name
                    .apply_as_prefix(&function.name, ItemType::Command);

                Ok(format!(
                    r#"
                {name}: command<{arg_type}, {ret_type}, {err_type}>("{full_key}")
                "#
                ))
            })
            .collect::<Result<Vec<_>, ExportError>>()?
            .join(",\n");

        Ok(formatdoc! {
            r#"
            export const commands = {{
            {commands}
            }}"#
        })
    }

    fn render_events(
        events: &[EventDataType],
        type_map: &TypeMap,
        cfg: &ExportConfig,
    ) -> Result<String, ExportError> {
        if events.is_empty() {
            return Ok(Default::default());
        }

        let (events_types, events_map) = js_ts::events_data(events, cfg, type_map)?;

        let events_types = events_types.join(",\n");

        Ok(formatdoc! {
            r#"
            export const events = __makeEvents__<{{
            {events_types}
            }}>({{
            {events_map}
            }})"#
        })
    }

    fn render(
        commands: &[FunctionDataType],
        events: &[EventDataType],
        type_map: &TypeMap,
        cfg: &ExportConfig,
    ) -> Result<String, ExportError> {
        let new_type_map = &mut TypeMap::default();

        specta::export::get_types().for_each(|f| new_type_map.insert(f.0, f.1));
        type_map
            .iter()
            .for_each(|f| new_type_map.insert(f.0, f.1.clone()));

        let dependant_types = new_type_map
            .iter()
            .map(|(_sid, ndt)| ts::export_named_datatype(&cfg.inner, ndt, new_type_map))
            .collect::<Result<Vec<_>, _>>()
            .map(|v| v.join("\n"))?;

        js_ts::render_all_parts::<Self>(
            commands,
            events,
            new_type_map,
            cfg,
            &dependant_types,
            GLOBALS,
        )
    }
}
