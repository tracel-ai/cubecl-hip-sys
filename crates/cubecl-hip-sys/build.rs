include!("src/hipconfig.rs");

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
        // Select the bindings for the detected HIP patch. When the detected patch is NEWER than any
        // bindings we ship (e.g. ROCm 7.13 reports patch 99004, past the newest shipped 53211), fall
        // back to the latest available bindings. This is ABI-safe: AMD versions every changed symbol
        // (`hipGetDevicePropertiesR0600`, `hipDeviceProp_tR0600`, ...), and ROCm 7.13's headers still
        // map `hipGetDeviceProperties` -> the R0600 revision, so the latest bindings link against the
        // newer runtime unchanged. This mirrors the no-hipconfig branch (which already clamps to the
        // latest bindings) instead of emitting a `hip_<patch>` feature that has no bindings module.
        let toml = std::fs::read_to_string("Cargo.toml")
            .expect("cubecl-hip-sys build.rs: failed to read Cargo.toml");
        let requested = format!("hip_{default_version}");
        let selected = if hip_feature_available(&requested, &toml) {
            requested
        } else {
            let latest = extract_latest_hip_feature_from_path("Cargo.toml")
                .expect("cubecl-hip-sys build.rs: no hip_<patch> bindings feature in Cargo.toml");
            println!(
                "cargo::warning=No cubecl-hip-sys bindings for detected HIP patch {default_version}; \
                 using the latest available bindings '{latest}' (ABI-compatible via AMD's versioned symbols)."
            );
            latest
        };
        println!("cargo:rustc-cfg=feature=\"{selected}\"");
    } else {
        panic!("Error: HIP_XXX feature detected!\nHIP_XXX features should not be set manually. Remove the feature and change your HIP_PATH environment variable instead.");
    }
}

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=ROCM_PATH");
    println!("cargo::rerun-if-env-changed=HIP_PATH");
    let hip_system_patch = get_hip_patch_version();
    if let Ok(ref patch) = hip_system_patch {
        set_hip_feature(patch);
        println!("cargo::rustc-link-lib=dylib=hiprtc");
        println!("cargo::rustc-link-lib=dylib=amdhip64");
        let lib_path = get_hip_ld_library_path().unwrap();
        println!("cargo::rustc-link-search=native={lib_path}");
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
