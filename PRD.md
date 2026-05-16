# Project Rewrite Ventoy in Rust

## Product Requirements Document (PRD)

### 1. Executive Summary
Ventoy is a popular open-source tool for creating bootable USB drives from multiple disk image files (ISO, WIM, IMG, VHD(x), EFI) without repeated formatting. The existing C-based codebase, while stable and feature-rich, faces challenges in maintainability, safety, and extensibility. This project aims to rewrite Ventoy fully in Rust, targeting the same feature set and performance, while leveraging Rust’s memory safety, concurrency, and modern ecosystem for long-term improvements.

### 2. Goals
- **Functional Parity:** Achieve complete feature parity with the current Ventoy 1.1.x release line, including all officially supported image formats, boot modes, plugins, and tools.
- **Improved Safety:** Eliminate whole classes of memory safety bugs (buffer overflows, use-after-free) inherent in C, leading to a more robust bootable USB creator and boot-time environment.
- **Maintainability &amp; Modularity:** Restructure the codebase into well-tested, reusable Rust crates, making it easier to add new features, support new OSes, and integrate with emerging firmware standards.
- **Performance:** Retain or exceed the current read/write speeds during USB preparation and ISO booting. Rust’s zero-cost abstractions should not introduce perceptible regressions.
- **Compatibility:** Ensure that:
  - USB drives prepared by the new Rust version are interoperable with drives created by the C version and vice versa.
  - Existing Ventoy plugin configurations, themes, and auto-install scripts continue to work without modification.
  - All supported BIOS/UEFI architectures (Legacy x86, IA32 UEFI, x86_64 UEFI, ARM64 UEFI, MIPS64EL UEFI) remain supported.

### 3. Target Audience
- Linux, Windows, and macOS users who need a multi-boot USB solution.
- System administrators and IT professionals deploying OS installations.
- Developers and enthusiasts who customize boot environments.
- Downstream integrators (e.g., LiveCD/Rescue distributions) who bundle Ventoy.

### 4. Functional Requirements (from existing Ventoy features)

#### 4.1 Core USB Disk Preparation
- **F1:** Install Ventoy to a USB stick, local disk, SSD, NVMe, or SD card without destroying existing data on secondary partitions (data non-destructive upgrade).
- **F2:** Support both MBR and GPT partition tables.
- **F3:** Allow the main data partition to be formatted with FAT32, exFAT, NTFS, UDF, XFS, Btrfs, Ext2/3/4.
- **F4:** Handle ISO files larger than 4 GB (e.g., Windows install images).
- **F5:** USB normal use unaffected: after Ventoy installation, the device remains usable as a regular storage device.

#### 4.2 Image Booting
- **F6:** Boot from image files (ISO, WIM, IMG, VHD, VHDX, EFI) placed directly on the main partition; no extraction needed.
- **F7:** Present a boot menu (Legacy &amp; UEFI native styles) listing all discovered image files. Support TreeView and List view modes.
- **F8:** Support browsing and booting image files stored on local disks (not just the USB).
- **F9:** Support memdisk mode (loading entire image into RAM).
- **F10:** Support for booting Linux vDisks (vtoyboot plugin).

#### 4.3 BIOS and UEFI Support
- **F11:** Legacy BIOS boot (x86).
- **F12:** UEFI boot for multiple architectures: IA32, x86_64, ARM64, MIPS64EL.
- **F13:** IA32/x86_64 UEFI Secure Boot support (enrollment and signing).

#### 4.4 Plugins &amp; Extensions
- **F14:** Plugin framework with a GUI configurator (VentoyPlugson).
- **F15:** Persistence support for Linux distributions (persistence image or partition).
- **F16:** Auto-installation scripts: unattended Windows installation (unattend.xml) and Linux preseed/kickstart/autoinstall.
- **F17:** Variable expansion in auto-install scripts.
- **F18:** Injection plugin: injecting files/drivers into the runtime environment.
- **F19:** Boot configuration file dynamic replacement (Conf Replace plugin).
- **F20:** Password protection for booting specific images.
- **F21:** DUD (Driver Update Disk) plugin.
- **F22:** Menu alias, menu tip, menu class customization.
- **F23:** Custom themes (GRUB2-like theme engine).

#### 4.5 Management Tools
- **F24:** Ventoy2Disk: CLI and GUI (Linux &amp; Windows) tools to install/update/uninstall Ventoy.
- **F25:** VentoyPlugson: GUI plugin configuration editor.
- **F26:** Ventoy LiveCD creation tool.
- **F27:** IPXE integration (VtoyTool).

### 5. Non-Functional Requirements

#### 5.1 Performance
- **NFR1:** USB preparation (format, install) must complete as fast as the current C implementation, limited only by device I/O.
- **NFR2:** Boot menu enumeration and image boot times should not regress.
- **NFR3:** Memory footprint of the boot-time components must remain small enough to run within the pre-OS environment (especially Legacy BIOS).

#### 5.2 Reliability &amp; Safety
- **NFR4:** No unsafe memory handling in Rust code except where necessary for FFI or low-level device access; all FFI blocks must be strictly audited.
- **NFR5:** All file system operations (partitioning, formatting, image mounting) must be atomic where possible and recoverable from interruption.

#### 5.3 Compatibility
- **NFR6:** Maintain backward compatibility with all Ventoy configuration files (ventoy.json, theme files, auto-install scripts).
- **NFR7:** Support the exact same disk layout and metadata structure so that a Rust-prepared USB works with any Ventoy-compatible system and vice versa.

#### 5.4 Portability
- **NFR8:** The Rust project must compile and run on major platforms: Linux (x86_64, aarch64), Windows (x86_64), and macOS (for USB preparation tools). Boot-time components may target only the specific firmware environments.

#### 5.5 Licensing &amp; Open Source
- **NFR9:** The project remains under GPL-3.0 or a compatible license, with consideration to the original C Ventoy’s GPL-3.0.
- **NFR10:** All dependencies shall have licenses compatible with GPL-3.0.

### 6. User Stories (Key Scenarios)
- As a user, I copy several Linux ISOs and a Windows 11 ISO to my USB drive, reboot, and see a boot menu listing them all. I select one and boot into the live environment or installer.
- As a sysadmin, I create an unattended Windows installation ISO by placing an autounattend.xml next to the ISO file on the Ventoy USB.
- As a developer, I customize the boot menu theme and add password protection for certain sensitive rescue ISOs.
- As a user running on an ARM64 server (e.g., Raspberry Pi 4), I create a Ventoy USB and boot an ARM64 Ubuntu Server image via UEFI.

### 7. Constraints &amp; Risks
- **C1:** The bootloader core (GRUB2 based in the original) is complex and deeply tied to the BIOS/UEFI environment. A pure Rust bootloader (e.g., using `rust-osdev` crates) may not yet be mature enough for all architectures. **Mitigation:** Initially, we may reuse parts of the original bootloader as a fallback or rewrite gradually, starting with the user-space tools and plugin logic.
- **C2:** Secure Boot signing infrastructure requires signing keys and shim integration; rewriting this in Rust must still comply with Microsoft’s UEFI signing requirements. **Mitigation:** Leverage existing shim and provide Rust-based tooling for key enrollment.
- **C3:** The huge variety of tested ISOs (&gt;1300) and special handling per distro must be preserved. **Mitigation:** Use the original compatibility database and test suite; build automated CI testing with QEMU for a representative subset.
- **C4:** Some low-level operations (partition table manipulation, direct disk I/O) require `unsafe` on Windows and Linux; must be carefully wrapped.

### 8. Success Metrics
- All documented features from Ventoy 1.1.12 pass the original test suite on a reference set of 200+ ISOs.
- The Rust implementation is accepted by community maintainers and at least 80% of active contributors agree to transition.
- No critical memory bugs reported in the Rust components over a 6-month stabilization period.
- Compilation warnings (e.g., with Clippy) are treated as errors and kept at zero.

---

## Plan for Rewriting Ventoy in Rust

### 1. High-Level Architecture
Instead of a monolithic C project, we will decompose Ventoy into a set of loosely coupled Rust crates:

- **`ventoy-core`** – Core library: partition layout logic, metadata serialization/deserialization, image discovery, file system abstraction.
- **`ventoy-bootloader`** – Boot-time components: BIOS/UEFI boot stub, GRUB2 integration or pure Rust bootloader, Secure Boot handling, memdisk, vDisk boot.
- **`ventoy-plugins`** – Plugin system, persistence, auto-install, injection, theme engine.
- **`ventoy-cli`** – Ventoy2Disk CLI tool (equivalent to current shell/C binary).
- **`ventoy-gui`** – GUI tools using a cross-platform framework (e.g., egui or Tauri) for Ventoy2Disk GUI, VentoyPlugson, and LiveCD GUI.
- **`ventoy-fs`** – File system drivers/mounting (FUSE, partition tools). May leverage existing Rust crates (e.g., `fatfs`, `ntfs`, `ext4`).
- **`ventoy-utils`** – Common helpers, logging, error handling.

Where necessary, we will keep thin C/assembly shims for bootloader entry points that still need to interact with firmware, but we aim to replace them over time.

### 2. Technology Stack
- **Language:** Rust (stable channel)
- **Build System:** Cargo with workspace
- **Cross-Platform GUI:** egui (with wgpu) for native look, or Tauri for web-based GUIs while preserving Rust backend.
- **Disk/Partitioning:** `gpt` crate, `partition-manager` (if suitable) or thin wrappers around OS-specific APIs (Windows: `DeviceIoControl`, Linux: `libblkid`, `libparted` FFI or pure Rust reimplementation).
- **File Systems:** Rust crates like `fatfs`, `exfat`, `ntfs3g` bindings (or pure Rust if available), `ext4` tools.
- **Cryptography:** `ring` or `rustls` for Secure Boot signing operations, key generation.
- **Virtualization Testing:** QEMU, `vmtest` crate for running boot tests in CI.
- **Packaging:** GitHub Actions CI building Linux (AppImage, deb, rpm), Windows (MSI/portable), macOS (dmg). Bootable artifacts distributed as raw images/EFI binaries.

### 3. Development Phases &amp; Milestones

#### Phase 0: Research &amp; Foundation (Months 1-2)
- Deep analysis of current Ventoy source code, documenting all partition layout variants, metadata structures, and boot flows.
- Evaluate Rust bootloader options: `rust-osdev` ecosystem, UEFI application development, Legacy BIOS compatibility.
- Define exact on-disk format specification (MBR/GPT partition layout, reserved sectors, secondary boot area).
- Set up a cross-compilation CI for target architectures (ARM64, MIPS64, etc.).

**Deliverables:** Detailed technical specification document, CI pipeline, `ventoy-core` crate with metadata reading/writing in pure Rust.

#### Phase 2: Core USB Preparation (Months 3-4)
- Implement partition creation (MBR &amp; GPT) using Rust libraries or safe wrappers.
- Implement Ventoy's unique disk layout: protective MBR/GPT, 1 MB gap, EFI system partition, main data partition.
- Support formatting the main partition with FAT32/exFAT/NTFS/Ext* (using OS-appropriate tools or integrated libraries).
- Non-destructive update mechanism.
- ISO file scanning and boot menu generation (static configuration) – initially in a simulation environment.

**Deliverables:** CLI tool `ventoy-cli` that can create a rudimentary Ventoy USB and list ISOs on it.

#### Phase 3: Bootloader Integration (Months 5-7)
- **Legacy BIOS:** Integrate with existing GRUB2 core image (as binary blob) while replacing surrounding scripts and loaders with Rust-generated configuration.
- **UEFI:** Write a minimal UEFI application in Rust that scans for Ventoy metadata (via serialized protobuf/JSON) and chainloads GRUB2 or a custom Rust boot manager that handles image mounting and booting.
- Implement memdisk mode (loading entire ISO into RAM) using standard firmware interfaces.
- Secure Boot enrollment tooling and signature verification in the Rust UEFI stub.

**Deliverables:** Bootable USB that launches a boot menu on both legacy BIOS and UEFI (x86_64) with basic ISO booting (Linux and Windows).

#### Phase 4: Plugins and Advanced Features (Months 8-10)
- Plugin framework: define a trait-based interface in Rust that plugins implement; serialize/deserialize compatible with `ventoy.json`.
- Implement persistence (overlay filesystem and partition backends).
- Auto-install script processing: parse unattend.xml/kickstart, inject into boot parameters, variable expansion.
- Theme engine (compatible with existing Ventoy theme format) using embedded graphics (unlikely in pre-boot, may generate GRUB themes directly).
- Password protection (integrated into menu).
- Injection (DUD) and Conf Replace.

**Deliverables:** Full feature parity with current Ventoy except possibly ARM64/MIPS UEFI; VentoyPlugson rewritten in Rust GUI.

#### Phase 5: Platform Expansion, Testing &amp; Polish (Months 11-12)
- Port UEFI bootloader to ARM64 and MIPS64EL using cross-compilation and QEMU testing.
- Comprehensive CI test suite covering 200+ ISOs across major distributions; regression tests for disk layout compatibility.
- GUI tool maturation (Ventoy2Disk GUI, LiveCD creator) using egui or Tauri.
- Documentation, migration guide for existing users, community engagement.
- Performance profiling and optimization.

**Deliverables:** Rust-based Ventoy v2.0.0 release candidate, officially replacing the C version on the GitHub repository.

### 4. Team &amp; Roles
- **System Programmers (2-3):** Focus on bootloader, kernel interactions, file systems.
- **Embedded/Rust devs (2):** UEFI/Legacy boot, firmware support.
- **GUI Developer (1):** Cross-platform VentoyPlugson and installer GUI.
- **QA/Test Engineer (1):** Build automated ISO boot test lab, maintain compatibility matrix.
- **Project Maintainer (1):** Overall architecture, release management, community liaison.

### 5. Risk Mitigation
- **GRUB2 Rewrite is too ambitious:** Keep GRUB2 as the boot manager for the foreseeable future, only rewrite the surrounding config generation, module management, and user tools. The Rust UEFI stub will be an alternative but may not be the default initially.
- **File system support diversity:** Instead of implementing all file system drivers in Rust, we can offload formatting to host tools (mkfs.fat, mkfs.ntfs) via subprocess when possible, focusing Rust on reading existing file systems for boot-time (which may still require libraries).
- **Secure Boot key management complexity:** Provide a helper that calls existing secure boot utilities (sbsign, mokutil) or integrates them as dependencies; avoid implementing from scratch unless needed.
- **Community resistance to Rust:** Publish incremental releases, show performance and safety benefits, maintain open development, and encourage contributions.

### 6. Continuous Improvement
- After release, collect feedback and refactor. Gradually increase the share of pure Rust components (e.g., replacing GRUB2 modules with Rust equivalents for specific file systems).
- Establish a `ventoy-rs` standalone repo initially, then merge into the main ventoy/Ventoy repository once stable.
