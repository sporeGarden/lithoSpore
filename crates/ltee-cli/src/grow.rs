// SPDX-License-Identifier: AGPL-3.0-or-later

//! `litho grow` — germinate a USB artifact into a full development environment.
//!
//! The USB carries everything needed for offline validation (Tier 1 + 2).
//! When plugged into a clean system with internet, `litho grow` clones the
//! source repo, installs the Rust toolchain, builds from source, fetches
//! all upstream data, and validates — turning the seed into a full tree.
//!
//! Stages:
//!   1. Clone source repo (from scope.toml [source] metadata)
//!   2. Detect or install Rust toolchain
//!   3. Build from source (cargo build --release)
//!   4. Seed data from USB into cloned tree
//!   5. Fetch remaining datasets
//!   6. Validate (Tier 1 + Tier 2)
//!   7. Optionally provision a benchScale VM

use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run(
    artifact_root: &str,
    target: &str,
    vm: bool,
    container: bool,
    ecosystem: bool,
    skip_build: bool,
    skip_fetch: bool,
) {
    let root = Path::new(artifact_root);
    let target_path = PathBuf::from(target);

    // Container-only mode: build OCI image and validate inside it.
    // No clone/build/fetch needed — the USB artifact is self-contained.
    if container {
        stage_container(root);
        return;
    }

    println!("=== lithoSpore grow — germinating hypogeal cotyledon ===");
    println!("  Artifact: {artifact_root}");
    println!("  Target:   {target}");
    println!();

    let scope = load_source_metadata(root);

    // Stage 1: Clone
    stage_clone(root, &target_path, &scope, ecosystem);

    // Stage 2: Toolchain
    if !skip_build {
        stage_toolchain(&scope);
    }

    // Stage 3: Build
    if !skip_build {
        stage_build(&target_path, &scope);
    }

    // Stage 4: Seed data from USB
    stage_seed_data(root, &target_path);

    // Stage 5: Fetch
    if !skip_fetch {
        stage_fetch(&target_path);
    }

    // Stage 6: Validate
    stage_validate(&target_path);

    // Stage 7: VM (optional)
    if vm {
        stage_vm(root);
    }

    println!();
    step("Germination Complete");
    println!("  Full source tree:  {target}");
    println!("  Build artifacts:   {}/target/release/", target_path.display());
    println!();
    println!("  To develop:  cd {target} && cargo test --workspace");
    println!("  To validate: cd {target} && cargo run --release --bin litho -- validate");
    if ecosystem {
        let eco = target_path.parent().unwrap_or(&target_path);
        println!("  Ecosystem:   {}", eco.display());
    }
}

struct SourceConfig {
    repo: String,
    repo_https: String,
    branch: String,
    ecosystem_repo: String,
    ecosystem_repo_https: String,
    rust_toolchain: String,
    rust_target: String,
}

fn load_source_metadata(root: &Path) -> SourceConfig {
    let scope_path = root.join("artifact/scope.toml");
    if let Ok(scope) = litho_core::ScopeManifest::load(&scope_path) {
        if let Some(src) = scope.source {
            return SourceConfig {
                repo: src.repo,
                repo_https: src.repo_https,
                branch: if src.branch.is_empty() { "main".into() } else { src.branch },
                ecosystem_repo: src.ecosystem_repo,
                ecosystem_repo_https: src.ecosystem_repo_https,
                rust_toolchain: if src.rust_toolchain.is_empty() { "stable".into() } else { src.rust_toolchain },
                rust_target: if src.rust_target.is_empty() { "x86_64-unknown-linux-musl".into() } else { src.rust_target },
            };
        }
    }

    eprintln!("  WARNING: No [source] metadata in scope.toml — using defaults");
    SourceConfig {
        repo: "https://github.com/sporeGarden/lithoSpore.git".into(),
        repo_https: "https://github.com/sporeGarden/lithoSpore.git".into(),
        branch: "main".into(),
        ecosystem_repo: "https://github.com/sporeGarden/ecoPrimals.git".into(),
        ecosystem_repo_https: "https://github.com/sporeGarden/ecoPrimals.git".into(),
        rust_toolchain: "stable".into(),
        rust_target: "x86_64-unknown-linux-musl".into(),
    }
}

fn stage_clone(_root: &Path, target: &Path, scope: &SourceConfig, ecosystem: bool) {
    step("1. Cloning source repository");

    if ecosystem {
        let eco_target = target.parent().unwrap_or(target);
        if eco_target.join(".git").exists() {
            println!("  Ecosystem repo already exists at {}", eco_target.display());
        } else {
            let repo_url = pick_git_url(&scope.ecosystem_repo, &scope.ecosystem_repo_https);
            println!("  Cloning ecosystem: {repo_url}");
            run_cmd("git", &["clone", "--depth", "1", "-b", &scope.branch, &repo_url,
                             &eco_target.to_string_lossy()]);
        }
        let garden_path = eco_target.join("gardens/lithoSpore");
        if garden_path.exists() && !target.exists() {
            println!("  lithoSpore found at {}", garden_path.display());
            println!("  NOTE: Use the ecosystem path for development.");
        }
    }

    if target.join(".git").exists() {
        println!("  Source repo already exists at {}", target.display());
        println!("  Pulling latest...");
        run_cmd_in("git", &["pull", "--ff-only"], target);
        return;
    }

    if target.exists() && std::fs::read_dir(target).map(|mut d| d.next().is_some()).unwrap_or(false) {
        println!("  Target directory is not empty and not a git repo.");
        println!("  Skipping clone — will attempt to use existing content.");
        return;
    }

    let repo_url = pick_git_url(&scope.repo, &scope.repo_https);
    println!("  Cloning: {repo_url} → {}", target.display());
    run_cmd("git", &["clone", "--depth", "1", "-b", &scope.branch, &repo_url,
                     &target.to_string_lossy()]);
    println!("  Source cloned successfully");
}

fn stage_toolchain(scope: &SourceConfig) {
    step("2. Checking Rust toolchain");

    if let Ok(output) = Command::new("rustc").arg("--version").output() {
        let version = String::from_utf8_lossy(&output.stdout);
        println!("  Found: {}", version.trim());
    } else {
        println!("  Rust not found — installing via rustup...");
        let status = Command::new("sh")
            .args(["-c", &format!("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain {}", scope.rust_toolchain)])
            .status();
        match status {
            Ok(s) if s.success() => println!("  Rust installed successfully"),
            Ok(s) => {
                eprintln!("  ERROR: rustup exited with {s}");
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("  ERROR: Could not run rustup installer: {e}");
                eprintln!("  Install manually: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh");
                std::process::exit(1);
            }
        }
    }

    if let Ok(output) = Command::new("rustup").args(["target", "list", "--installed"]).output() {
        let installed = String::from_utf8_lossy(&output.stdout);
        if !installed.contains(&scope.rust_target) {
            println!("  Adding target: {}", scope.rust_target);
            run_cmd("rustup", &["target", "add", &scope.rust_target]);
        } else {
            println!("  Target {} already installed", scope.rust_target);
        }
    }

    if which("musl-gcc") {
        println!("  musl-tools: available");
    } else {
        println!("  WARNING: musl-gcc not found — `apt install musl-tools` or equivalent");
        println!("           Build will proceed without musl (non-static binary)");
    }
}

fn stage_build(target: &Path, scope: &SourceConfig) {
    step("3. Building from source");

    if !target.join("Cargo.toml").exists() {
        eprintln!("  ERROR: No Cargo.toml found at {}", target.display());
        eprintln!("  Cannot build — clone may have failed.");
        return;
    }

    let mut args = vec!["build", "--release"];
    if which("musl-gcc") {
        args.extend(["--target", &scope.rust_target]);
        println!("  Building: cargo {} (musl-static)", args.join(" "));
    } else {
        println!("  Building: cargo {} (native)", args.join(" "));
    }

    run_cmd_in("cargo", &args, target);

    let binary = if which("musl-gcc") {
        target.join(format!("target/{}/release/litho", scope.rust_target))
    } else {
        target.join("target/release/litho")
    };
    if binary.exists() {
        let size = std::fs::metadata(&binary).map(|m| m.len()).unwrap_or(0);
        println!("  Built: {} ({:.1} MB)", binary.display(), size as f64 / 1_048_576.0);
    } else {
        eprintln!("  WARNING: Expected binary not found at {}", binary.display());
    }
}

fn stage_seed_data(usb_root: &Path, target: &Path) {
    step("4. Seeding data from USB artifact");

    let usb_data = usb_root.join("artifact/data");
    let target_data = target.join("artifact/data");

    if !usb_data.exists() {
        println!("  No USB data to seed");
        return;
    }

    std::fs::create_dir_all(&target_data).ok();
    let mut seeded = 0u32;

    for entry in walkdir::WalkDir::new(&usb_data).into_iter().filter_map(|e| e.ok()) {
        let rel = match entry.path().strip_prefix(&usb_data) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let dest = target_data.join(rel);

        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&dest).ok();
        } else if !dest.exists() {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            if std::fs::copy(entry.path(), &dest).is_ok() {
                seeded += 1;
            }
        }
    }

    for toml_name in ["scope.toml", "data.toml", "tolerances.toml"] {
        let src = usb_root.join(format!("artifact/{toml_name}"));
        let dst = target.join(format!("artifact/{toml_name}"));
        if src.exists() && !dst.exists() {
            std::fs::copy(&src, &dst).ok();
        }
    }

    let usb_expected = usb_root.join("validation/expected");
    let target_expected = target.join("validation/expected");
    if usb_expected.exists() {
        std::fs::create_dir_all(&target_expected).ok();
        for entry in walkdir::WalkDir::new(&usb_expected).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let rel = entry.path().strip_prefix(&usb_expected).unwrap_or(entry.path());
                let dest = target_expected.join(rel);
                if !dest.exists() {
                    std::fs::copy(entry.path(), &dest).ok();
                    seeded += 1;
                }
            }
        }
    }

    println!("  Seeded {seeded} files from USB into cloned tree");
}

fn stage_fetch(target: &Path) {
    step("5. Fetching upstream datasets");

    let litho = find_litho_binary(target);
    if let Some(bin) = &litho {
        println!("  Using: {}", bin.display());
        let status = Command::new(bin)
            .args(["fetch", "--all", "--artifact-root", "."])
            .current_dir(target)
            .status();
        match status {
            Ok(s) if s.success() => println!("  Fetch complete"),
            Ok(s) => eprintln!("  WARNING: fetch exited with {s}"),
            Err(e) => eprintln!("  WARNING: Could not run litho fetch: {e}"),
        }
    } else {
        println!("  No litho binary found — running cargo to fetch");
        let status = Command::new("cargo")
            .args(["run", "--release", "--bin", "litho", "--", "fetch", "--all"])
            .current_dir(target)
            .status();
        match status {
            Ok(s) if s.success() => println!("  Fetch complete"),
            _ => eprintln!("  WARNING: cargo run fetch failed"),
        }
    }

    detect_sra_toolkit();
}

fn stage_validate(target: &Path) {
    step("6. Validating grown tree");

    let litho = find_litho_binary(target);
    let runner: Box<dyn FnMut(&[&str]) -> Option<std::process::ExitStatus>> = if let Some(ref bin) = litho {
        let bin = bin.clone();
        let target = target.to_path_buf();
        Box::new(move |args: &[&str]| {
            Command::new(&bin).args(args)
                .arg("--artifact-root").arg(".")
                .current_dir(&target)
                .status().ok()
        })
    } else {
        let target = target.to_path_buf();
        Box::new(move |args: &[&str]| {
            let mut cmd_args = vec!["run", "--release", "--bin", "litho", "--"];
            cmd_args.extend_from_slice(args);
            Command::new("cargo").args(&cmd_args)
                .current_dir(&target)
                .status().ok()
        })
    };

    let mut runner = runner;

    println!("  Running Tier 2 (Rust) validation...");
    if let Some(s) = runner(&["validate", "--max-tier", "2"]) {
        if s.success() {
            println!("  Tier 2: PASS");
        } else {
            eprintln!("  Tier 2: FAIL (exit {})", s.code().unwrap_or(-1));
        }
    }

    println!("  Running Tier 1 (Python) validation...");
    if let Some(s) = runner(&["validate", "--max-tier", "1"]) {
        if s.success() {
            println!("  Tier 1: PASS");
        } else {
            eprintln!("  Tier 1: FAIL (exit {})", s.code().unwrap_or(-1));
        }
    }
}

fn stage_container(root: &Path) {
    step("Container Deployment — benchScale substrate");

    let runtime = detect_container_runtime();
    let runtime = match runtime {
        Some(r) => r,
        None => {
            eprintln!("  ERROR: No container runtime found.");
            eprintln!("  Install Docker:  https://docs.docker.com/get-docker/");
            eprintln!("  Install Podman:  https://podman.io/getting-started/installation");
            std::process::exit(1);
        }
    };
    println!("  Runtime: {runtime}");

    let containerfile = root.join("Containerfile");
    if !containerfile.exists() {
        eprintln!("  ERROR: No Containerfile found at {}", containerfile.display());
        eprintln!("  The USB artifact should contain a Containerfile at its root.");
        std::process::exit(1);
    }

    let image_name = "litho-spore:local";
    println!("  Building image: {image_name}");
    let status = Command::new(&runtime)
        .args(["build", "-f", "Containerfile", "-t", image_name, "."])
        .current_dir(root)
        .status();
    match status {
        Ok(s) if s.success() => println!("  Image built successfully"),
        Ok(s) => {
            eprintln!("  ERROR: {runtime} build exited with {s}");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("  ERROR: Could not run `{runtime} build`: {e}");
            std::process::exit(1);
        }
    }

    println!();
    println!("  Running Tier 2 (Rust) validation in container...");
    let t2 = Command::new(&runtime)
        .args(["run", "--rm", image_name,
               "validate", "--artifact-root", "/opt/lithoSpore", "--max-tier", "2"])
        .status();
    match t2 {
        Ok(s) if s.success() => println!("  Tier 2: PASS"),
        Ok(s) => eprintln!("  Tier 2: FAIL (exit {})", s.code().unwrap_or(-1)),
        Err(e) => eprintln!("  Tier 2: ERROR ({e})"),
    }

    println!("  Running Tier 1 (Python) validation in container...");
    let t1 = Command::new(&runtime)
        .args(["run", "--rm", image_name,
               "validate", "--artifact-root", "/opt/lithoSpore", "--max-tier", "1"])
        .status();
    match t1 {
        Ok(s) if s.success() => println!("  Tier 1: PASS"),
        Ok(s) => eprintln!("  Tier 1: FAIL (exit {})", s.code().unwrap_or(-1)),
        Err(e) => eprintln!("  Tier 1: ERROR ({e})"),
    }

    println!("  Running data verification in container...");
    let verify = Command::new(&runtime)
        .args(["run", "--rm", image_name,
               "verify", "--artifact-root", "/opt/lithoSpore"])
        .status();
    match verify {
        Ok(s) if s.success() => println!("  Verify: PASS"),
        Ok(s) => eprintln!("  Verify: FAIL (exit {})", s.code().unwrap_or(-1)),
        Err(e) => eprintln!("  Verify: ERROR ({e})"),
    }

    println!();
    step("Container Deployment Complete");
    println!("  Image:     {image_name}");
    println!("  Substrate: {runtime}");
    println!();
    println!("  Interactive:      {runtime} run -it --entrypoint /bin/bash {image_name}");
    println!("  Airgap mode:      {runtime} run --rm --network=none {image_name}");
    println!("  Custom command:   {runtime} run --rm {image_name} self-test --artifact-root /opt/lithoSpore");
}

fn detect_container_runtime() -> Option<String> {
    for candidate in ["docker", "podman"] {
        if which(candidate) {
            return Some(candidate.to_string());
        }
    }
    None
}

fn stage_vm(root: &Path) {
    step("7. Provisioning benchScale VM");

    if !which("virt-install") {
        eprintln!("  SKIP: virt-install not found.");
        eprintln!("  Install: apt install libvirt-daemon-system virtinst qemu-system-x86 genisoimage");
        println!();
        println!("  Alternative: use --container for Docker/Podman deployment (any OS).");
        return;
    }

    let cloud_init = root.join("scripts/vm-cloud-init.yaml");
    if !cloud_init.exists() {
        eprintln!("  SKIP: No cloud-init template at {}", cloud_init.display());
        return;
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
    let cloud_image_candidates = [
        PathBuf::from("/var/lib/libvirt/images/ubuntu-24.04-server-cloudimg-amd64.img"),
        PathBuf::from(&home).join("images/ubuntu-24.04-server-cloudimg-amd64.img"),
        PathBuf::from("/tmp/ubuntu-24.04-server-cloudimg-amd64.img"),
    ];
    let cloud_image = cloud_image_candidates.iter().find(|p| p.exists());

    if cloud_image.is_none() {
        println!("  No cloud image found. Downloading Ubuntu 24.04 cloud image...");
        let dl_path = &cloud_image_candidates[2];
        let status = Command::new("curl")
            .args(["-fSL", "-o", &dl_path.to_string_lossy(),
                   "https://cloud-images.ubuntu.com/releases/24.04/release/ubuntu-24.04-server-cloudimg-amd64.img"])
            .status();
        match status {
            Ok(s) if s.success() => println!("  Downloaded: {}", dl_path.display()),
            _ => {
                eprintln!("  ERROR: Could not download cloud image.");
                eprintln!("  Manually place ubuntu-24.04-server-cloudimg-amd64.img in /tmp/");
                return;
            }
        }
    }
    let cloud_image = cloud_image_candidates.iter().find(|p| p.exists()).unwrap();

    let vm_name = "litho-grow-vm";
    let vm_disk = format!("/var/lib/libvirt/images/{vm_name}.qcow2");
    let cidata_iso = format!("/tmp/{vm_name}-cidata.iso");

    // Clean up any previous attempt
    let _ = Command::new("virsh").args(["destroy", vm_name])
        .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status();
    let _ = Command::new("virsh").args(["undefine", vm_name, "--remove-all-storage"])
        .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status();

    // Build cloud-init ISO
    println!("  Building cloud-init ISO...");
    let cidata_dir = std::env::temp_dir().join(format!("{vm_name}-cidata"));
    std::fs::create_dir_all(&cidata_dir).ok();
    std::fs::write(
        cidata_dir.join("meta-data"),
        "instance-id: litho-grow-001\nlocal-hostname: litho-vm\n",
    ).ok();
    std::fs::copy(&cloud_init, cidata_dir.join("user-data")).ok();

    let iso_tool = if which("genisoimage") { "genisoimage" } else if which("mkisofs") { "mkisofs" } else {
        eprintln!("  ERROR: genisoimage or mkisofs required for cloud-init ISO");
        eprintln!("  Install: apt install genisoimage");
        return;
    };
    let status = Command::new(iso_tool)
        .args(["-output", &cidata_iso, "-volid", "cidata", "-joliet", "-rock",
               &cidata_dir.to_string_lossy()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    let _ = std::fs::remove_dir_all(&cidata_dir);
    if !status.map(|s| s.success()).unwrap_or(false) {
        eprintln!("  ERROR: Failed to build cloud-init ISO");
        return;
    }

    // Create VM disk from cloud image
    println!("  Preparing VM disk...");
    let _ = std::fs::copy(cloud_image, &vm_disk);
    let _ = Command::new("qemu-img").args(["resize", &vm_disk, "20G"])
        .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status();

    // Launch VM
    println!("  Launching VM: {vm_name} (2 vCPU, 4 GB RAM, 20 GB disk)");
    let status = Command::new("virt-install")
        .args([
            "--name", vm_name,
            "--memory", "4096",
            "--vcpus", "2",
            "--disk", &format!("path={vm_disk},format=qcow2"),
            "--disk", &format!("path={cidata_iso},device=cdrom"),
            "--os-variant", "ubuntu24.04",
            "--network", "network=default",
            "--graphics", "none",
            "--noautoconsole",
            "--import",
        ])
        .stdout(std::process::Stdio::null())
        .status();
    if !status.map(|s| s.success()).unwrap_or(false) {
        eprintln!("  ERROR: virt-install failed");
        return;
    }

    // Wait for IP
    println!("  Waiting for VM to boot...");
    let mut vm_ip = String::new();
    for _ in 0..60 {
        if let Ok(output) = Command::new("virsh").args(["domifaddr", vm_name]).output() {
            let text = String::from_utf8_lossy(&output.stdout);
            if let Some(ip) = text.split_whitespace()
                .find(|w| w.contains('.') && w.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false))
            {
                vm_ip = ip.split('/').next().unwrap_or(ip).to_string();
                break;
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }

    if vm_ip.is_empty() {
        eprintln!("  ERROR: Could not discover VM IP address");
        println!("  VM is running — check manually: virsh domifaddr {vm_name}");
        return;
    }

    println!("  VM booted: {vm_ip}");
    println!("  Cloud-init will clone, build, and validate inside the VM.");
    println!("  Monitor: ssh litho@{vm_ip} tail -f /var/log/litho-grow.log");
    println!();
    println!("  Cleanup: virsh destroy {vm_name} && virsh undefine {vm_name} --remove-all-storage");
}

fn step(msg: &str) {
    println!();
    println!("=== {msg} ===");
}

fn pick_git_url(ssh: &str, https: &str) -> String {
    if !ssh.is_empty() && which("ssh") {
        if Command::new("ssh")
            .args(["-o", "BatchMode=yes", "-o", "ConnectTimeout=5", "-T", "git@github.com"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.code() == Some(1))
            .unwrap_or(false)
        {
            return ssh.to_string();
        }
    }
    if !https.is_empty() {
        return https.to_string();
    }
    ssh.to_string()
}

fn which(cmd: &str) -> bool {
    Command::new("which").arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn run_cmd(cmd: &str, args: &[&str]) {
    let status = Command::new(cmd).args(args).status();
    match status {
        Ok(s) if !s.success() => {
            eprintln!("  ERROR: `{cmd} {}` exited with {s}", args.join(" "));
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("  ERROR: Could not run `{cmd}`: {e}");
            std::process::exit(1);
        }
        _ => {}
    }
}

fn run_cmd_in(cmd: &str, args: &[&str], dir: &Path) {
    let status = Command::new(cmd).args(args).current_dir(dir).status();
    match status {
        Ok(s) if !s.success() => {
            eprintln!("  ERROR: `{cmd} {}` in {} exited with {s}", args.join(" "), dir.display());
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("  ERROR: Could not run `{cmd}` in {}: {e}", dir.display());
            std::process::exit(1);
        }
        _ => {}
    }
}

fn find_litho_binary(target: &Path) -> Option<PathBuf> {
    for candidate in [
        target.join("target/x86_64-unknown-linux-musl/release/litho"),
        target.join("target/release/litho"),
        target.join("bin/litho"),
    ] {
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

fn detect_sra_toolkit() {
    if which("prefetch") && which("fastq-dump") {
        println!("  SRA toolkit detected — genomic data fetch available");
        println!("  Use `prefetch PRJNA*` + `fastq-dump` for full NCBI datasets");
    }
}
