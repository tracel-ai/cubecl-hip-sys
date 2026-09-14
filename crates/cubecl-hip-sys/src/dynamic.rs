use libloading::{Library, Symbol};
use std::{env, path::PathBuf, sync::OnceLock};

struct Libraries {
    hip: Library,
    hiprtc: Library,
}

static LIBRARIES: OnceLock<Result<Libraries, String>> = OnceLock::new();

/// Returns whether both HIP runtime libraries can be loaded.
pub fn is_available() -> bool {
    libraries().is_ok()
}

/// Resolve a HIP symbol without making the HIP libraries link-time dependencies.
///
/// This function is called by the generated bindings. A missing runtime is a
/// runtime error because the public binding functions cannot return one common
/// error type for all of their C signatures.
pub(crate) unsafe fn load<T: Copy>(name: &[u8]) -> T {
    let libraries = libraries()
        .as_ref()
        .unwrap_or_else(|error| panic!("{error}"));
    let library = if name.starts_with(b"hiprtc") {
        &libraries.hiprtc
    } else {
        &libraries.hip
    };

    let symbol: Symbol<'_, T> = unsafe { library.get(name) }.unwrap_or_else(|error| {
        let symbol_name = name.strip_suffix(&[0]).unwrap_or(name);
        let symbol = String::from_utf8_lossy(symbol_name);
        panic!("HIP symbol `{symbol}` is unavailable: {error}");
    });
    *symbol
}

fn libraries() -> &'static Result<Libraries, String> {
    LIBRARIES.get_or_init(|| unsafe { load_libraries() })
}

unsafe fn load_libraries() -> Result<Libraries, String> {
    let search_paths = search_paths();
    let hip = load_library("amdhip64", &search_paths)?;
    let hiprtc = load_library("hiprtc", &search_paths)?;
    Ok(Libraries { hip, hiprtc })
}

fn search_paths() -> Vec<PathBuf> {
    ["ROCM_PATH", "HIP_PATH"]
        .into_iter()
        .filter_map(|variable| env::var_os(variable))
        .flat_map(|path| {
            let path = PathBuf::from(path);
            [path.join("lib"), path]
        })
        .collect()
}

unsafe fn load_library(name: &str, search_paths: &[PathBuf]) -> Result<Library, String> {
    let names = library_names(name);
    let mut errors = Vec::new();

    for path in search_paths {
        for library_name in &names {
            let candidate = path.join(library_name);
            match unsafe { Library::new(&candidate) } {
                Ok(library) => return Ok(library),
                Err(error) => errors.push(format!("{}: {error}", candidate.display())),
            }
        }
    }

    for library_name in &names {
        match unsafe { Library::new(library_name) } {
            Ok(library) => return Ok(library),
            Err(error) => errors.push(format!("{library_name}: {error}")),
        }
    }

    Err(format!(
        "Could not load HIP library `{name}`. Install ROCm or set ROCM_PATH/HIP_PATH.\n{}",
        errors.join("\n")
    ))
}

fn library_names(name: &str) -> Vec<String> {
    if cfg!(target_os = "windows") {
        vec![format!("{name}.dll")]
    } else if cfg!(target_os = "macos") {
        vec![format!("lib{name}.dylib")]
    } else {
        vec![
            format!("lib{name}.so"),
            format!("lib{name}.so.1"),
            format!("lib{name}.so.0"),
        ]
    }
}
