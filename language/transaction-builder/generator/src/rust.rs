// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common;
use diem_types::transaction::{ArgumentABI, ScriptABI, TypeArgumentABI};
use move_core_types::language_storage::TypeTag;
use serde_generate::{
    indent::{IndentConfig, IndentedWriter},
    rust, CodeGeneratorConfig,
};

use heck::{CamelCase, ShoutySnakeCase};
use std::{
    collections::BTreeMap,
    io::{Result, Write},
    path::PathBuf,
};

/// Output transaction builders in Rust for the given ABIs.
/// If `local_types` is true, we generate a file suitable for the Diem codebase itself
/// rather than using serde-generated, standalone definitions.
pub fn output(out: &mut dyn Write, abis: &[ScriptABI], local_types: bool) -> Result<()> {
    let mut emitter = RustEmitter {
        out: IndentedWriter::new(out, IndentConfig::Space(4)),
        local_types,
    };

    emitter.output_preamble()?;
    emitter.output_script_call_enum_with_imports(abis)?;

    writeln!(emitter.out, "\nimpl ScriptCall {{")?;
    emitter.out.indent();
    emitter.output_encode_method(abis)?;
    emitter.output_decode_method()?;
    emitter.out.unindent();
    writeln!(emitter.out, "\n}}")?;

    for abi in abis {
        emitter.output_script_encoder_function(abi)?;
    }

    for abi in abis {
        emitter.output_script_decoder_function(abi)?;
    }

    emitter.output_decoder_map(abis)?;
    emitter.output_decoding_helpers(abis)?;

    for abi in abis {
        emitter.output_code_constant(abi)?;
    }
    Ok(())
}

/// Shared state for the Rust code generator.
struct RustEmitter<T> {
    /// Writer.
    out: IndentedWriter<T>,
    /// Whether we are targetting the Diem repository itself (as opposed to generated Diem types).
    local_types: bool,
}

impl<T> RustEmitter<T>
where
    T: Write,
{
    fn output_preamble(&mut self) -> Result<()> {
        if self.local_types {
            writeln!(
                self.out,
                r#"
// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// This file was generated. Do not modify!
//
// To re-generate this code, run: `cargo run --release -p stdlib`
"#
            )?;
        }
        writeln!(
            self.out,
            r#"//! Conversion diemry between a structured representation of a Move script call (`ScriptCall`) and the
//! standard BCS-compatible representation used in Diem transactions (`Script`).
//!
//! This code was generated by compiling known Script interfaces ("ABIs") with the tool `transaction-builder-generator`.
"#
        )
    }

    fn output_script_call_enum_with_imports(&mut self, abis: &[ScriptABI]) -> Result<()> {
        let external_definitions = Self::get_external_definitions(self.local_types);
        let script_registry: BTreeMap<_, _> = vec![(
            "ScriptCall".to_string(),
            common::make_abi_enum_container(abis),
        )]
        .into_iter()
        .collect();
        let mut comments: BTreeMap<_, _> = abis
            .iter()
            .map(|abi| {
                (
                    vec![
                        "crate".to_string(),
                        "ScriptCall".to_string(),
                        abi.name().to_camel_case(),
                    ],
                    common::prepare_doc_string(abi.doc()),
                )
            })
            .collect();
        comments.insert(
            vec!["crate".to_string(), "ScriptCall".to_string()],
            r#"Structured representation of a call into a known Move script.
```ignore
impl ScriptCall {
    pub fn encode(self) -> Script { .. }
    pub fn decode(&Script) -> Option<ScriptCall> { .. }
}
```
"#
            .into(),
        );
        let custom_derive_block = if self.local_types {
            Some(
                r#"#[cfg_attr(any(test, feature = "fuzzing"), derive(proptest_derive::Arbitrary))]
#[cfg_attr(any(test, feature = "fuzzing"), proptest(no_params))]"#
                    .to_string(),
            )
        } else {
            None
        };
        // Deactivate serialization for local types to force `Bytes = Vec<u8>`.
        let config = CodeGeneratorConfig::new("crate".to_string())
            .with_comments(comments)
            .with_external_definitions(external_definitions)
            .with_serialization(!self.local_types);
        rust::CodeGenerator::new(&config)
            .with_custom_derive_block(custom_derive_block)
            .output(&mut self.out, &script_registry)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, format!("{}", err)))?;
        Ok(())
    }

    fn get_external_definitions(local_types: bool) -> serde_generate::ExternalDefinitions {
        let definitions = if local_types {
            vec![
                ("move_core_types::language_storage", vec!["TypeTag"]),
                (
                    "diem_types::transaction",
                    vec!["Script", "TransactionArgument"],
                ),
                ("diem_types::account_address", vec!["AccountAddress"]),
            ]
        } else {
            vec![(
                "diem_types",
                vec!["AccountAddress", "TypeTag", "Script", "TransactionArgument"],
            )]
        };
        definitions
            .into_iter()
            .map(|(module, defs)| {
                (
                    module.to_string(),
                    defs.into_iter().map(String::from).collect(),
                )
            })
            .collect()
    }

    fn output_encode_method(&mut self, abis: &[ScriptABI]) -> Result<()> {
        writeln!(
            self.out,
            r#"
/// Build a Diem `Script` from a structured object `ScriptCall`.
pub fn encode(self) -> Script {{"#
        )?;
        self.out.indent();
        writeln!(self.out, "use ScriptCall::*;\nmatch self {{")?;
        self.out.indent();
        for abi in abis {
            self.output_variant_encoder(abi)?;
        }
        self.out.unindent();
        writeln!(self.out, "}}")?;
        self.out.unindent();
        writeln!(self.out, "}}\n")
    }

    fn output_variant_encoder(&mut self, abi: &ScriptABI) -> Result<()> {
        let params = std::iter::empty()
            .chain(abi.ty_args().iter().map(TypeArgumentABI::name))
            .chain(abi.args().iter().map(ArgumentABI::name))
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(
            self.out,
            "{0}{{{2}}} => encode_{1}_script({2}),",
            abi.name().to_camel_case(),
            abi.name(),
            params
        )
    }

    fn output_decode_method(&mut self) -> Result<()> {
        writeln!(
            self.out,
            r#"
/// Try to recognize a Diem `Script` and convert it into a structured object `ScriptCall`.
pub fn decode(script: &Script) -> Option<ScriptCall> {{
    match SCRIPT_DECODER_MAP.get({}) {{
        Some(decoder) => decoder(script),
        None => None,
    }}
}}"#,
            if self.local_types {
                "script.code()"
            } else {
                "&script.code.clone().into_vec()"
            }
        )
    }

    fn output_comment(&mut self, indentation: usize, doc: &str) -> std::io::Result<()> {
        let prefix = " ".repeat(indentation) + "/// ";
        let empty_line = "\n".to_string() + &" ".repeat(indentation) + "///\n";
        let text = textwrap::indent(doc, &prefix).replace("\n\n", &empty_line);
        write!(self.out, "\n{}", text)
    }

    fn output_script_encoder_function(&mut self, abi: &ScriptABI) -> Result<()> {
        self.output_comment(0, &common::prepare_doc_string(abi.doc()))?;
        write!(
            self.out,
            "pub fn encode_{}_script({}) -> Script {{",
            abi.name(),
            [
                Self::quote_type_parameters(abi.ty_args()),
                Self::quote_parameters(abi.args(), self.local_types),
            ]
            .concat()
            .join(", ")
        )?;
        self.out.indent();
        if self.local_types {
            writeln!(
                self.out,
                r#"
Script::new(
    {}_CODE.to_vec(),
    vec![{}],
    vec![{}],
)"#,
                abi.name().to_shouty_snake_case(),
                Self::quote_type_arguments(abi.ty_args()),
                Self::quote_arguments(abi.args()),
            )?;
        } else {
            writeln!(
                self.out,
                r#"
Script {{
    code: Bytes::from({}_CODE.to_vec()),
    ty_args: vec![{}],
    args: vec![{}],
}}"#,
                abi.name().to_shouty_snake_case(),
                Self::quote_type_arguments(abi.ty_args()),
                Self::quote_arguments(abi.args()),
            )?;
        }
        self.out.unindent();
        writeln!(self.out, "}}")?;
        Ok(())
    }

    fn output_script_decoder_function(&mut self, abi: &ScriptABI) -> Result<()> {
        writeln!(
            self.out,
            "\nfn decode_{}_script({}script: &Script) -> Option<ScriptCall> {{",
            abi.name(),
            // fix warning "unused variable"
            if abi.ty_args().is_empty() && abi.args().is_empty() {
                "_"
            } else {
                ""
            }
        )?;
        self.out.indent();
        writeln!(
            self.out,
            "Some(ScriptCall::{} {{",
            abi.name().to_camel_case(),
        )?;
        self.out.indent();
        for (index, ty_arg) in abi.ty_args().iter().enumerate() {
            writeln!(
                self.out,
                "{} : script.ty_args{}.get({})?.clone(),",
                ty_arg.name(),
                if self.local_types { "()" } else { "" },
                index,
            )?;
        }
        for (index, arg) in abi.args().iter().enumerate() {
            writeln!(
                self.out,
                "{} : decode_{}_argument(script.args{}.get({})?.clone())?,",
                arg.name(),
                common::mangle_type(arg.type_tag()),
                if self.local_types { "()" } else { "" },
                index,
            )?;
        }
        self.out.unindent();
        writeln!(self.out, "}})")?;
        self.out.unindent();
        writeln!(self.out, "}}")?;
        Ok(())
    }

    fn output_decoder_map(&mut self, abis: &[ScriptABI]) -> Result<()> {
        writeln!(
            self.out,
            r#"
type DecoderMap = std::collections::HashMap<Vec<u8>, Box<dyn Fn(&Script) -> Option<ScriptCall> + std::marker::Sync + std::marker::Send>>;

static SCRIPT_DECODER_MAP: once_cell::sync::Lazy<DecoderMap> = once_cell::sync::Lazy::new(|| {{"#
        )?;
        self.out.indent();
        writeln!(
            self.out,
            "let mut map : DecoderMap = std::collections::HashMap::new();"
        )?;
        for abi in abis {
            writeln!(
                self.out,
                "map.insert({}_CODE.to_vec(), Box::new(decode_{}_script));",
                abi.name().to_shouty_snake_case(),
                abi.name()
            )?;
        }
        writeln!(self.out, "map")?;
        self.out.unindent();
        writeln!(self.out, "}});")
    }

    fn output_decoding_helpers(&mut self, abis: &[ScriptABI]) -> Result<()> {
        let required_types = common::get_required_decoding_helper_types(abis);
        for required_type in required_types {
            self.output_decoding_helper(required_type)?;
        }
        Ok(())
    }

    fn output_decoding_helper(&mut self, type_tag: &TypeTag) -> Result<()> {
        use TypeTag::*;
        let (constructor, expr) = match type_tag {
            Bool => ("Bool", "Some(value)".to_string()),
            U8 => ("U8", "Some(value)".to_string()),
            U64 => ("U64", "Some(value)".to_string()),
            U128 => ("U128", "Some(value)".to_string()),
            Address => ("Address", "Some(value)".to_string()),
            Vector(type_tag) => match type_tag.as_ref() {
                U8 => ("U8Vector", "Some(value)".to_string()),
                _ => common::type_not_allowed(type_tag),
            },
            Struct(_) | Signer => common::type_not_allowed(type_tag),
        };
        writeln!(
            self.out,
            r#"
fn decode_{}_argument(arg: TransactionArgument) -> Option<{}> {{
    match arg {{
        TransactionArgument::{}(value) => {},
        _ => None,
    }}
}}
"#,
            common::mangle_type(type_tag),
            Self::quote_type(type_tag, self.local_types),
            constructor,
            expr,
        )
    }

    fn output_code_constant(&mut self, abi: &ScriptABI) -> Result<()> {
        writeln!(
            self.out,
            "\nconst {}_CODE: &[u8] = &[{}];",
            abi.name().to_shouty_snake_case(),
            abi.code()
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        Ok(())
    }

    fn quote_type_parameters(ty_args: &[TypeArgumentABI]) -> Vec<String> {
        ty_args
            .iter()
            .map(|ty_arg| format!("{}: TypeTag", ty_arg.name()))
            .collect()
    }

    fn quote_parameters(args: &[ArgumentABI], local_types: bool) -> Vec<String> {
        args.iter()
            .map(|arg| {
                format!(
                    "{}: {}",
                    arg.name(),
                    Self::quote_type(arg.type_tag(), local_types)
                )
            })
            .collect()
    }

    fn quote_type_arguments(ty_args: &[TypeArgumentABI]) -> String {
        ty_args
            .iter()
            .map(|ty_arg| ty_arg.name().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn quote_arguments(args: &[ArgumentABI]) -> String {
        args.iter()
            .map(|arg| Self::quote_transaction_argument(arg.type_tag(), arg.name()))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn quote_type(type_tag: &TypeTag, local_types: bool) -> String {
        use TypeTag::*;
        match type_tag {
            Bool => "bool".into(),
            U8 => "u8".into(),
            U64 => "u64".into(),
            U128 => "u128".into(),
            Address => "AccountAddress".into(),
            Vector(type_tag) => match type_tag.as_ref() {
                U8 => {
                    if local_types {
                        "Vec<u8>".into()
                    } else {
                        "Bytes".into()
                    }
                }
                _ => common::type_not_allowed(type_tag),
            },

            Struct(_) | Signer => common::type_not_allowed(type_tag),
        }
    }

    fn quote_transaction_argument(type_tag: &TypeTag, name: &str) -> String {
        use TypeTag::*;
        match type_tag {
            Bool => format!("TransactionArgument::Bool({})", name),
            U8 => format!("TransactionArgument::U8({})", name),
            U64 => format!("TransactionArgument::U64({})", name),
            U128 => format!("TransactionArgument::U128({})", name),
            Address => format!("TransactionArgument::Address({})", name),
            Vector(type_tag) => match type_tag.as_ref() {
                U8 => format!("TransactionArgument::U8Vector({})", name),
                _ => common::type_not_allowed(type_tag),
            },

            Struct(_) | Signer => common::type_not_allowed(type_tag),
        }
    }
}

pub struct Installer {
    install_dir: PathBuf,
    diem_types_version: String,
}

impl Installer {
    pub fn new(install_dir: PathBuf, diem_types_version: String) -> Self {
        Installer {
            install_dir,
            diem_types_version,
        }
    }
}

impl crate::SourceInstaller for Installer {
    type Error = Box<dyn std::error::Error>;

    fn install_transaction_builders(
        &self,
        public_name: &str,
        abis: &[ScriptABI],
    ) -> std::result::Result<(), Self::Error> {
        let (name, version) = {
            let parts = public_name.splitn(2, ':').collect::<Vec<_>>();
            if parts.len() >= 2 {
                (parts[0].to_string(), parts[1].to_string())
            } else {
                (parts[0].to_string(), "0.1.0".to_string())
            }
        };
        let dir_path = self.install_dir.join(&name);
        std::fs::create_dir_all(&dir_path)?;
        let mut cargo = std::fs::File::create(&dir_path.join("Cargo.toml"))?;
        write!(
            cargo,
            r#"[package]
name = "{}"
version = "{}"
edition = "2018"

[dependencies]
once_cell = "1.4.0"
serde = {{ version = "1.0", features = ["derive"] }}
serde_bytes = "0.11"
diem-types = {{ path = "../diem-types", version = "{}" }}
"#,
            name, version, self.diem_types_version,
        )?;
        std::fs::create_dir(dir_path.join("src"))?;
        let source_path = dir_path.join("src/lib.rs");
        let mut source = std::fs::File::create(&source_path)?;
        output(&mut source, abis, /* local_types */ false)?;
        Ok(())
    }
}
