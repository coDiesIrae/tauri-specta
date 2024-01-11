use crate::{js_ts, *};
use heck::ToLowerCamelCase;
use indoc::formatdoc;
use specta::{
    functions::FunctionDataType,
    js_doc,
    ts::{self, ExportError},
    TypeMap,
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

                        ts::datatype(&cfg.inner, r, type_map)
                    }
                    t => ts::datatype(&cfg.inner, t, type_map),
                }?;

                let err_type = match &function.result {
                    specta::DataType::Result(t) => {
                        let (_, e) = t.as_ref();

                        ts::datatype(&cfg.inner, e, type_map)
                    }
                    _ => Ok("void".to_string()),
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
        _type_map: &TypeMap,
        cfg: &ExportConfig,
    ) -> Result<String, ExportError> {
        let type_map = &mut TypeMap::default();

        specta::export::get_types().for_each(|f| type_map.insert(f.0, f.1));

        let dependant_types = type_map
            .iter()
            .map(|(_sid, ndt)| ts::export_named_datatype(&cfg.inner, ndt, type_map))
            .collect::<Result<Vec<_>, _>>()
            .map(|v| v.join("\n"))?;

        js_ts::render_all_parts::<Self>(commands, events, type_map, cfg, &dependant_types, GLOBALS)
    }
}
