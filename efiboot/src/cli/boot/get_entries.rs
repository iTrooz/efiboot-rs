use efivar::{boot::BootEntryAttributes, VarManager};

pub fn get_entries(manager: Box<dyn VarManager>, verbose: bool) {
    let entries = match manager.get_boot_entries() {
        Ok(entries) => entries,
        Err(err) => {
            eprintln!("Failed to get boot entries: {}", err);
            return;
        }
    };

    println!("Boot entries (in boot order):");

    for entry in entries {
        println!("--");
        println!("Description: {:?}", entry.description);
        println!(
            "Enabled: {:?}",
            entry
                .attributes
                .contains(BootEntryAttributes::LOAD_OPTION_ACTIVE)
        );

        println!(
            "Boot file: {}",
            entry
                .file_path_list
                .map(|fpl| fpl.to_string())
                .unwrap_or_else(|| "None/Invalid".to_owned())
        );

        if verbose {
            if !entry.optional_data.is_empty() {
                println!(
                    "Optional data: {}",
                    entry
                        .optional_data
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<Vec<String>>()
                        .join(" ")
                );
            }

            if !entry.attributes.is_empty() {
                println!("Attributes: {}", entry.attributes);
            }
        }
    }
}
