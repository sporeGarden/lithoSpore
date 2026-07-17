// SPDX-License-Identifier: AGPL-3.0-or-later

use super::util::{step, which};
use std::path::{Path, PathBuf};
use std::process::Command;

const DEFAULT_LIBVIRT_IMAGES: &str = "/var/lib/libvirt/images";

/// Libvirt image storage directory (`LITHO_LIBVIRT_IMAGES` overrides the default).
fn libvirt_images_dir() -> PathBuf {
    std::env::var(litho_core::env_vars::LITHO_LIBVIRT_IMAGES)
        .map_or_else(|_| PathBuf::from(DEFAULT_LIBVIRT_IMAGES), PathBuf::from)
}

pub(super) fn stage_container(root: &Path) {
    step("Container Deployment — benchScale substrate");

    let runtime = detect_container_runtime();
    let runtime = if let Some(r) = runtime {
        r
    } else {
        eprintln!("  ERROR: No container runtime found.");
        eprintln!("  Install Docker:  https://docs.docker.com/get-docker/");
        eprintln!("  Install Podman:  https://podman.io/getting-started/installation");
        std::process::exit(1);
    };
    println!("  Runtime: {runtime}");

    let containerfile = root.join("Containerfile");
    if !containerfile.exists() {
        eprintln!(
            "  ERROR: No Containerfile found at {}",
            containerfile.display()
        );
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
        .args([
            "run",
            "--rm",
            image_name,
            "validate",
            "--artifact-root",
            "/opt/lithoSpore",
            "--max-tier",
            "2",
        ])
        .status();
    match t2 {
        Ok(s) if s.success() => println!("  Tier 2: PASS"),
        Ok(s) => eprintln!("  Tier 2: FAIL (exit {})", s.code().unwrap_or(-1)),
        Err(e) => eprintln!("  Tier 2: ERROR ({e})"),
    }

    println!("  Running Tier 1 (Python) validation in container...");
    let t1 = Command::new(&runtime)
        .args([
            "run",
            "--rm",
            image_name,
            "validate",
            "--artifact-root",
            "/opt/lithoSpore",
            "--max-tier",
            "1",
        ])
        .status();
    match t1 {
        Ok(s) if s.success() => println!("  Tier 1: PASS"),
        Ok(s) => eprintln!("  Tier 1: FAIL (exit {})", s.code().unwrap_or(-1)),
        Err(e) => eprintln!("  Tier 1: ERROR ({e})"),
    }

    println!("  Running data verification in container...");
    let verify = Command::new(&runtime)
        .args([
            "run",
            "--rm",
            image_name,
            "verify",
            "--artifact-root",
            "/opt/lithoSpore",
        ])
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
    println!(
        "  Custom command:   {runtime} run --rm {image_name} self-test --artifact-root /opt/lithoSpore"
    );
}

fn detect_container_runtime() -> Option<String> {
    for candidate in ["docker", "podman"] {
        if which(candidate) {
            return Some(candidate.to_string());
        }
    }
    None
}

pub(super) fn stage_vm(root: &Path) {
    step("7. Provisioning benchScale VM");

    if !which("virt-install") {
        eprintln!("  SKIP: virt-install not found.");
        eprintln!(
            "  Install: apt install libvirt-daemon-system virtinst qemu-system-x86 genisoimage"
        );
        println!();
        println!("  Alternative: use --container for Docker/Podman deployment (any OS).");
        return;
    }

    let cloud_init = root.join("scripts/vm-cloud-init.yaml");
    if !cloud_init.exists() {
        eprintln!("  SKIP: No cloud-init template at {}", cloud_init.display());
        return;
    }

    let home = std::env::var(litho_core::env_vars::HOME).unwrap_or_else(|_| "/root".into());
    let libvirt_images = libvirt_images_dir();
    let tmp_cloud_image = std::env::temp_dir().join("ubuntu-24.04-server-cloudimg-amd64.img");
    let cloud_image_candidates = [
        libvirt_images.join("ubuntu-24.04-server-cloudimg-amd64.img"),
        PathBuf::from(&home).join("images/ubuntu-24.04-server-cloudimg-amd64.img"),
        tmp_cloud_image,
    ];
    let cloud_image = cloud_image_candidates.iter().find(|p| p.exists());

    if cloud_image.is_none() {
        println!("  No cloud image found. Downloading Ubuntu 24.04 cloud image...");
        let dl_path = &cloud_image_candidates[2];
        let status = Command::new("curl")
            .args([
                "-fSL",
                "-o",
                &dl_path.to_string_lossy(),
                litho_core::consts::VM_CLOUD_IMAGE_URL,
            ])
            .status();
        match status {
            Ok(s) if s.success() => println!("  Downloaded: {}", dl_path.display()),
            _ => {
                eprintln!("  ERROR: Could not download cloud image.");
                eprintln!(
                    "  Manually place ubuntu-24.04-server-cloudimg-amd64.img in {}",
                    std::env::temp_dir().display()
                );
                return;
            }
        }
    }
    let Some(cloud_image) = cloud_image_candidates.iter().find(|p| p.exists()) else {
        eprintln!("  ERROR: No cloud image found after download attempt.");
        return;
    };

    let vm_name = "litho-grow-vm";
    let vm_disk = libvirt_images_dir().join(format!("{vm_name}.qcow2"));
    let vm_disk_s = vm_disk.to_string_lossy();
    let cidata_iso = std::env::temp_dir().join(format!("{vm_name}-cidata.iso"));

    // Clean up any previous attempt
    let _ = Command::new("virsh")
        .args(["destroy", vm_name])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    let _ = Command::new("virsh")
        .args(["undefine", vm_name, "--remove-all-storage"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    // Build cloud-init ISO
    println!("  Building cloud-init ISO...");
    let cidata_dir = std::env::temp_dir().join(format!("{vm_name}-cidata"));
    std::fs::create_dir_all(&cidata_dir).ok();
    std::fs::write(
        cidata_dir.join("meta-data"),
        "instance-id: litho-grow-001\nlocal-hostname: litho-vm\n",
    )
    .ok();
    std::fs::copy(&cloud_init, cidata_dir.join("user-data")).ok();

    let iso_tool = if which("genisoimage") {
        "genisoimage"
    } else if which("mkisofs") {
        "mkisofs"
    } else {
        eprintln!("  ERROR: genisoimage or mkisofs required for cloud-init ISO");
        eprintln!("  Install: apt install genisoimage");
        return;
    };
    let cidata_iso_s = cidata_iso.to_string_lossy();
    let status = Command::new(iso_tool)
        .args([
            "-output",
            cidata_iso_s.as_ref(),
            "-volid",
            "cidata",
            "-joliet",
            "-rock",
            &cidata_dir.to_string_lossy(),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    let _ = std::fs::remove_dir_all(&cidata_dir);
    if !status.is_ok_and(|s| s.success()) {
        eprintln!("  ERROR: Failed to build cloud-init ISO");
        return;
    }

    // Create VM disk from cloud image
    println!("  Preparing VM disk...");
    let _ = std::fs::copy(cloud_image, vm_disk.as_path());
    let _ = Command::new("qemu-img")
        .args([
            "resize",
            vm_disk_s.as_ref(),
            litho_core::consts::VM_DISK_SIZE,
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    // Launch VM
    println!(
        "  Launching VM: {vm_name} ({} vCPU, {} MB RAM, {} disk)",
        litho_core::consts::VM_VCPU_COUNT,
        litho_core::consts::VM_RAM_MB,
        litho_core::consts::VM_DISK_SIZE,
    );
    let status = Command::new("virt-install")
        .args([
            "--name",
            vm_name,
            "--memory",
            litho_core::consts::VM_RAM_MB,
            "--vcpus",
            litho_core::consts::VM_VCPU_COUNT,
            "--disk",
            &format!("path={},format=qcow2", vm_disk.display()),
            "--disk",
            &format!("path={},device=cdrom", cidata_iso.display()),
            "--os-variant",
            litho_core::consts::VM_OS_VARIANT,
            "--network",
            "network=default",
            "--graphics",
            "none",
            "--noautoconsole",
            "--import",
        ])
        .stdout(std::process::Stdio::null())
        .status();
    if !status.is_ok_and(|s| s.success()) {
        eprintln!("  ERROR: virt-install failed");
        return;
    }

    // Wait for IP
    println!("  Waiting for VM to boot...");
    let mut vm_ip = String::new();
    for _ in 0..litho_core::consts::VM_BOOT_POLL_ATTEMPTS {
        if let Ok(output) = Command::new("virsh").args(["domifaddr", vm_name]).output() {
            let text = String::from_utf8_lossy(&output.stdout);
            if let Some(ip) = text
                .split_whitespace()
                .find(|w| w.contains('.') && w.chars().next().is_some_and(|c| c.is_ascii_digit()))
            {
                vm_ip = ip.split('/').next().unwrap_or(ip).to_string();
                break;
            }
        }
        std::thread::sleep(litho_core::consts::VM_BOOT_POLL_INTERVAL);
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
