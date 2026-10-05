// SPDX-FileCopyrightText: 2026 Veydan Project
// SPDX-License-Identifier: LicenseRef-PolyForm-Perimeter-1.0.1

//! Veydan Pass: the product crate (docs/platform-spec.md 13.1). It holds
//! the Tauri config, the strings of the product and the list of its
//! modules — one, the crate of pass; `veydan_shell` starts it, and the plan
//! of its sync is the shell's default over what the module registers
//! (9.1). The identifier `net.veydan.pass` and the version (`VERSION`,
//! written by `scripts/set-version.sh pass`) are in the Tauri config.
//!
//! One library for every platform: a computer and a phone run the same
//! module; the phone reads the QR code of a TOTP secret with the camera.

use veydan_shell::{Module, Product};

/// What Pass is (13.1, 13.5).
fn product() -> Product {
    Product {
        id: "pass",
        name: "Veydan Pass",
        desktop_entry: "veydanpass",
        icon: "veydanpass",
        sync: veydan_shell::default_plan(&module_list()),
    }
}

/// The modules of Pass, as `products.json` lists them.
pub fn module_list() -> Vec<Module> {
    vec![veydan_pass::module()]
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    veydan_shell::run(tauri::generate_context!(), product(), module_list());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The modules of the crate are those `products.json` gives the
    /// product: the UI is built from that list, the app from this one.
    #[test]
    fn the_module_list_is_that_of_products_json() {
        let manifest: serde_json::Value =
            serde_json::from_str(include_str!("../../../products.json")).unwrap();
        let listed: Vec<&str> = manifest["targets"]["pass"]["modules"]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| id.as_str().unwrap())
            .collect();
        let ours: Vec<&str> = module_list().iter().map(|module| module.id).collect();
        assert_eq!(ours, listed);
    }

    /// `commands.golden.txt`: every command of the product on a line with
    /// the module that answers it and where it exists (`all` platforms or
    /// `desktop` only), the lines of Space's file that belong to the shell
    /// and to pass. A name neither appears nor disappears without a
    /// matching change in the UI.
    #[test]
    fn every_command_has_the_owner_the_golden_file_names() {
        let expected: Vec<(&str, &str)> = include_str!("commands.golden.txt")
            .lines()
            .map(|line| {
                let mut words = line.split(' ');
                let (command, module, platform) = (
                    words.next().unwrap(),
                    words.next().unwrap(),
                    words.next().unwrap(),
                );
                assert!(matches!(platform, "all" | "desktop"), "{line}");
                (command, module, platform)
            })
            .filter(|(_, _, platform)| cfg!(desktop) || *platform == "all")
            .map(|(command, module, _)| (command, module))
            .collect();
        let table = veydan_shell::command_table(product(), module_list()).unwrap();
        for line in &table {
            assert!(expected.contains(line), "not in the golden file: {line:?}");
        }
        for line in &expected {
            assert!(table.contains(line), "the router does not know {line:?}");
        }
        assert_eq!(table.len(), expected.len());
    }

    /// The default plan takes everything the module and the shell register,
    /// in the order of 9.1: the system rows, then the rows of pass as the
    /// module registers them, the labels last. Pass has no handler, and a
    /// phone syncs no settings and no history of the generator.
    #[test]
    fn the_plan_applies_the_rows_of_pass_after_the_system_ones() {
        let registry = veydan_shell::sync_registry(product(), module_list()).unwrap();
        let (puts, _) = registry.apply_order();
        #[cfg(desktop)]
        let expected = [
            "password_vault",
            "setting",
            "totp",
            "password",
            "pw_history",
            "label",
        ];
        #[cfg(mobile)]
        let expected = ["password_vault", "totp", "password", "label"];
        assert_eq!(puts, expected);
        assert!(registry.handler_names().is_empty());
    }

    /// The data file of Pass: the core's, the lock's and sync's tables,
    /// and pass's. No table of notes, browser or ssh.
    #[test]
    fn the_schemas_are_the_cores_and_passs() {
        let mut names: Vec<_> = veydan_shell::schemas(&module_list())
            .iter()
            .map(|schema| schema.module)
            .collect();
        names.sort_unstable();
        assert_eq!(names, ["core", "lock", "pass", "sync"]);
    }
}
