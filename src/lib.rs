//! # Keystring generator
//! This package is intended to be used in cargo build-scripts.
//! It can be used to generate constant strings, that are used as keys in maps, configurations, etc.

use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::ops::Not;
use std::path::PathBuf;

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug)]
struct KeyElement {
    name: String,
    children: Vec<KeyElement>,
}

impl KeyElement {
    fn create_key(&mut self, key: &str) {
        let (key, remaining) = key.split_once(".").unwrap_or((key, ""));

        if self.children.iter().any(|c| c.name == key).not() {
            let mut child = KeyElement {
                name: key.to_string(),
                children: vec![],
            };

            if remaining.is_empty().not() {
                child.create_key(remaining);
            }

            self.children.push(child);
        } else if remaining.is_empty().not() {
            let children = &mut self.children;
            children.iter_mut()
                .find(|c| c.name == key)
                .unwrap()
                .create_key(remaining)
        }
    }

    fn generate_code(&self, separator: &str, parent: &str) -> Result<String, String> {
        let parent_string: String;
        if parent.is_empty() {
            parent_string = self.name.to_string();
        } else {
            parent_string = format!("{}{}{}", parent, separator, self.name);
        }
        if self.children.is_empty() {
            Ok(format!("pub const {}: &str = \"{}\";", self.name, parent_string))
        } else {
            let child_generated = self.children
                .iter()
                .map(|c| c.generate_code(separator, &parent_string).unwrap())
                .collect::<Vec<String>>()
                .join("");
            Ok(format!("pub mod {} {{pub const _BASE : &str = \"{}\";{} }}", self.name, parent_string, child_generated))
        }
    }
}

/// Generates rust source code from the given input file and saves it to the file `generated/keygen/keygen.rs`.
///
/// This function generates the code with a standard configuration. For examples and more configuration options see `generate_with_config`.
pub fn generate(input: &PathBuf) -> Result<(), String> {
    generate_with_config(input, None, false, ".")
}

/// Generates rust source code from the given input file.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use keystring_generator::generate_with_config;
/// let input_file = PathBuf::new().join("src/keygen/input.keys");
/// generate_with_config(
///     &input_file,
///     None,
///     true,
///     "."
/// ).unwrap();
/// ```
///
/// # Parameters
/// The following parameters can be supplied to this function:
///  * `input` - Path to the input file in any format as specified in `README.md`
///  * `output_dir` - Directory where the output file is generated. The output file will alyways be named `keygen.rs`.
///    The necessary directories will be created.
///    If `None` is supplied the default value (`generated/keygen`) will be used.
///  * `enable_warnings` - Whether the generated code should trigger warnings, like naming-conventions or unused code. If set to `false`, those warnings will be ignored.
///  * `separator` - Separator to use in the generated constants (e.g. `"."`, `":"`, `"/"`).
pub fn generate_with_config(
    input: &PathBuf,
    output_dir: Option<&PathBuf>,
    enable_warnings: bool,
    separator: &str,
) -> Result<(), String> {
    let mut input_file = File::open(input.as_path()).unwrap();
    let mut input_str = "".to_string();
    input_file.read_to_string(&mut input_str).unwrap();

    let compiled = compile_input(&input_str).unwrap();
    let output = compiled.iter()
        .map(|k| k.generate_code(separator, "").unwrap())
        .collect::<Vec<String>>()
        .join("\n");

    let control_macros: &str;
    if enable_warnings {
        control_macros = "";
    } else {
        control_macros = "#![allow(dead_code)]\n#![allow(non_upper_case_globals)]\n";
    }

    let default_pathbuf = PathBuf::new().join("generated/keygen");
    let out_path = output_dir
        .unwrap_or(&default_pathbuf);
    create_dir_all(out_path.as_path()).unwrap();
    let mut out_file = File::create(out_path.join("keygen.rs")).unwrap();
    out_file.write_all(control_macros.as_bytes()).unwrap();
    out_file.write_all(output.as_bytes()).unwrap();
    Ok(())
}

fn compile_input(input: &str) -> Result<Vec<KeyElement>, String> {
    let lines = input.lines();

    let mut root = KeyElement {
        name: "".to_string(),
        children: vec![],
    };
    let mut previous_line = "".to_string();
    let mut current_indentation = 0;
    let mut current_parent = "".to_string();
    let mut indentations = vec![];

    for ln in lines {
        let indent = count_leading_whitespaces(ln);
        let key = ln.trim_start().to_string();

        if indent > current_indentation {
            indentations.push((current_indentation, current_parent.to_string()));
            current_indentation = indent;
            if current_parent.is_empty() {
                current_parent = previous_line.to_string();
            } else {
                current_parent = current_parent + "." + &previous_line;
            }
        } else if indent < current_indentation {
            let mut restore = indentations.pop().unwrap();
            while restore.0 != indent {
                restore = indentations.pop().unwrap();

                if restore.0 < indent {
                    return Err("Illegal indentation in line \"".to_string() + ln + "\"!");
                }
            }

            current_indentation = restore.0;
            current_parent = restore.1;
        }

        if current_parent.is_empty() {
            root.create_key(&key);
        } else {
            root.create_key(&(current_parent.to_string() + "." + &key));
        }

        previous_line = key;
    }

    Ok(root.children)
}

fn count_leading_whitespaces(line: &str) -> usize {
    let replaced = line.replace("\t", "    ");
    let unindented = replaced.trim_start();
    replaced.len() - unindented.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hierarchical_input_compiles() {
        let input = include_str!("test/hierarchical.keys");
        assert_eq!(expecded_structure(), compile_input(input).unwrap());
    }

    #[test]
    fn enumerated_input_compiles() {
        let input = include_str!("test/enumerated.keys");
        assert_eq!(expecded_structure(), compile_input(input).unwrap());
    }

    #[test]
    fn mixed_input_compiles() {
        let input = include_str!("test/mixed.keys");
        assert_eq!(expecded_structure(), compile_input(input).unwrap());
    }

    fn expecded_structure() -> Vec<KeyElement> {
        vec![KeyElement {
            name: "hierarchical".to_string(),
            children: vec![
                KeyElement {
                    name: "keys".to_string(),
                    children: vec![
                        KeyElement {
                            name: "with".to_string(),
                            children: vec![
                                KeyElement {
                                    name: "five".to_string(),
                                    children: vec![
                                        KeyElement {
                                            name: "layers".to_string(),
                                            children: vec![],
                                        }
                                    ],
                                },
                                KeyElement {
                                    name: "six".to_string(),
                                    children: vec![
                                        KeyElement {
                                            name: "hierarchical".to_string(),
                                            children: vec![
                                                KeyElement {
                                                    name: "layers".to_string(),
                                                    children: vec![],
                                                }
                                            ],
                                        }
                                    ],
                                },
                            ],
                        }
                    ],
                }
            ],
        }]
    }
}
