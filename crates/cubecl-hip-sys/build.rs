include!("src/hipconfig.rs");

use std::{fs, path::Path};

const HIP_FEATURE_PREFIX: &str = "CARGO_FEATURE_HIP_";

/// Make sure that at least one and only one hip feature is set.
/// If None are set then we use the passed default version to set the corresponding feature.
/// Returns the selected HIP patch version.
fn set_hip_feature(default_version: &str) {
    let mut enabled_features = Vec::new();

    for (key, value) in std::env::vars() {
        if key.starts_with(HIP_FEATURE_PREFIX) && value == "1" {
            enabled_features.push(format!(
                "hip_{}",
                key.strip_prefix(HIP_FEATURE_PREFIX).unwrap()
            ));
        }
    }

    if enabled_features.is_empty() {
        let default_hip_feature = format!("hip_{default_version}");
        println!("cargo:rustc-cfg=feature=\"{default_hip_feature}\"");
    } else {
        panic!("Error: HIP_XXX feature detected!\nHIP_XXX features should not be set manually. Remove the feature and change your HIP_PATH environment variable instead.");
    }
}

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    generate_dynamic_bindings().expect("Should generate dynamic HIP bindings");
    println!("cargo::rerun-if-env-changed=ROCM_PATH");
    println!("cargo::rerun-if-env-changed=HIP_PATH");
    let hip_system_patch = get_hip_patch_version();
    if let Ok(ref patch) = hip_system_patch {
        set_hip_feature(patch);
    } else {
        // There is no 'hipconfig' on the system, so we assume there is no HIP installation available on the system.
        // Nevertheless we still want crates that depend on the bindings to compile even if they don't need to
        // link against the HIP libraries, especially for cargo clippy.
        // We decide to set the last version of HIP bindings as the default for this purpose, i.e. the HIP version that
        // corresponds to last published version of 'cubecl-hip-sys'.
        let feature = extract_latest_hip_feature_from_path("Cargo.toml").unwrap();
        println!(
            "cargo::warning=Defaulting to the latest feature of HIP bindings available: {feature}"
        );
        println!("cargo:rustc-cfg=feature=\"{feature}\"");
    }
}

/// Convert HIP declarations into wrappers that resolve their symbols at runtime.
///
/// The generated bindings also contain declarations for a few libc functions. Those are left
/// untouched; only HIP-prefixed declarations are rewritten.
fn generate_dynamic_bindings() -> std::io::Result<()> {
    let out_dir = Path::new(&std::env::var_os("OUT_DIR").expect("OUT_DIR is set")).to_path_buf();

    for entry in fs::read_dir("src/bindings")? {
        let entry = entry?;
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !file_name.starts_with("bindings_")
            || path.extension().and_then(|ext| ext.to_str()) != Some("rs")
        {
            continue;
        }

        println!("cargo::rerun-if-changed={}", path.display());
        let source = fs::read_to_string(&path)?;
        fs::write(out_dir.join(file_name), rewrite_bindings(&source))?;
    }

    Ok(())
}

fn rewrite_bindings(source: &str) -> String {
    let marker = "extern \"C\" {";
    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;

    while let Some(relative_start) = source[cursor..].find(marker) {
        let start = cursor + relative_start;
        let body_start = start + marker.len();
        let body_end = matching_brace(source, body_start - 1);

        output.push_str(&source[cursor..body_start]);
        let (body, wrappers) = rewrite_extern_block(&source[body_start..body_end]);
        output.push_str(&body);
        output.push('}');
        output.push_str(&wrappers);
        cursor = body_end + 1;
    }

    output.push_str(&source[cursor..]);
    output
}

fn matching_brace(source: &str, opening: usize) -> usize {
    let mut depth = 0;
    let mut in_string = false;
    let mut escaped = false;
    for (offset, character) in source[opening..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }
        if character == '"' {
            in_string = true;
            continue;
        }
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return opening + offset;
                }
            }
            _ => {}
        }
    }
    panic!("Unclosed extern block in generated bindings")
}

fn rewrite_extern_block(body: &str) -> (String, String) {
    let mut retained = String::with_capacity(body.len());
    let mut wrappers = String::new();
    let mut cursor = 0;
    let mut search = 0;

    while let Some(relative_start) = body[search..].find("pub fn ") {
        let function_start = search + relative_start;
        let name_start = function_start + "pub fn ".len();
        let name_end = body[name_start..]
            .find(|character: char| !character.is_ascii_alphanumeric() && character != '_')
            .map(|offset| name_start + offset)
            .expect("HIP binding function has a name");
        let name = &body[name_start..name_end];

        if !name.starts_with("hip") && !name.starts_with("__hip") {
            search = name_end;
            continue;
        }

        let opening = body[name_end..]
            .find('(')
            .map(|offset| name_end + offset)
            .expect("HIP binding function has arguments");
        let closing = matching_paren(body, opening);
        let semicolon = body[closing..]
            .find(';')
            .map(|offset| closing + offset)
            .expect("HIP binding function has a terminating semicolon");
        let attribute_start = preceding_attributes(body, function_start);
        let parameter_text = &body[opening + 1..closing];
        let return_type = body[closing + 1..semicolon].trim();
        let (parameter_types, parameter_names) = parameters(parameter_text);

        retained.push_str(&body[cursor..attribute_start]);
        wrappers.push('\n');
        wrappers.push_str(&body[attribute_start..function_start]);
        wrappers.push_str(&format!(
            "pub unsafe extern \"C\" fn {name}({parameter_text}) {return_type} {{\n    let __function: unsafe extern \"C\" fn({parameter_types}) {return_type} =\n        crate::dynamic::load(b\"{name}\\0\");\n    __function({parameter_names})\n}}\n"
        ));

        cursor = semicolon + 1;
        search = cursor;
    }

    retained.push_str(&body[cursor..]);
    (retained, wrappers)
}

fn matching_paren(source: &str, opening: usize) -> usize {
    let mut depth = 0;
    for (offset, character) in source[opening..].char_indices() {
        match character {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return opening + offset;
                }
            }
            _ => {}
        }
    }
    panic!("Unclosed HIP function parameter list")
}

fn preceding_attributes(source: &str, function_start: usize) -> usize {
    let mut line_start = source[..function_start]
        .rfind('\n')
        .map_or(0, |offset| offset + 1);
    loop {
        if line_start == 0 {
            return line_start;
        }
        let previous_line_end = line_start - 1;
        let previous_line_start = source[..previous_line_end]
            .rfind('\n')
            .map_or(0, |offset| offset + 1);
        if source[previous_line_start..previous_line_end]
            .trim_start()
            .starts_with("#[")
        {
            line_start = previous_line_start;
        } else {
            return line_start;
        }
    }
}

fn parameters(parameters: &str) -> (String, String) {
    let mut types = Vec::new();
    let mut names = Vec::new();
    let mut start = 0;
    let mut depth = 0;

    for (offset, character) in parameters.char_indices() {
        match character {
            '(' | '[' | '<' => depth += 1,
            ')' | ']' | '>' => depth -= 1,
            ',' if depth == 0 => {
                add_parameter(&parameters[start..offset], &mut types, &mut names);
                start = offset + 1;
            }
            _ => {}
        }
    }
    add_parameter(&parameters[start..], &mut types, &mut names);

    (types.join(", "), names.join(", "))
}

fn add_parameter<'a>(parameter: &'a str, types: &mut Vec<&'a str>, names: &mut Vec<&'a str>) {
    let parameter = parameter.trim();
    if parameter.is_empty() {
        return;
    }
    let colon = parameter
        .find(':')
        .expect("HIP binding parameter has a type");
    names.push(parameter[..colon].trim());
    types.push(parameter[colon + 1..].trim());
}
