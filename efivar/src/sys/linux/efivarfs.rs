use std::fs;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::str::FromStr;

use super::LinuxSystemManager;
use crate::efi::{VariableFlags, VariableName};
use crate::{Error, VarEnumerator, VarManager, VarReader, VarWriter};

pub const EFIVARFS_ROOT: &str = "/sys/firmware/efi/vars";

pub struct SystemManager;

impl SystemManager {
    pub fn new() -> SystemManager {
        SystemManager {}
    }
}

impl LinuxSystemManager for SystemManager {
    #[cfg(test)]
    fn supported(&self) -> bool {
        fs::metadata(EFIVARFS_ROOT).is_ok()
    }
}

impl VarEnumerator for SystemManager {
    fn get_var_names<'a>(&'a self) -> crate::Result<Box<dyn Iterator<Item = VariableName> + 'a>> {
        fs::read_dir(EFIVARFS_ROOT)
            .map(|list| {
                list.filter_map(Result::ok)
                    .filter(|entry| match entry.file_type() {
                        Ok(file_type) => file_type.is_dir(),
                        _ => false,
                    })
                    .filter_map(|entry| {
                        entry
                            .file_name()
                            .into_string()
                            .map_err(|_str| Error::InvalidUTF8)
                            .and_then(|s| VariableName::from_str(&s))
                            .ok()
                    })
            })
            .map(|it| -> Box<dyn Iterator<Item = VariableName>> { Box::new(it) })
            .map_err(|error| {
                // TODO: check for specific error types
                Error::UnknownIoError { error }
            })
    }
}

impl VarReader for SystemManager {
    fn read(&self, name: &VariableName) -> crate::Result<(Vec<u8>, VariableFlags)> {
        // Path to the attributes file
        let attributes_filename = format!("{}/{}/attributes", EFIVARFS_ROOT, name);

        // Open attributes file
        let f =
            File::open(attributes_filename).map_err(|error| Error::for_variable(error, name))?;
        let reader = BufReader::new(&f);

        let mut flags = VariableFlags::empty();
        for line in reader.lines() {
            let line = line.map_err(|error| Error::for_variable(error, name))?;
            let parsed = VariableFlags::from_str(&line)?;
            flags |= parsed;
        }

        // Filename to the matching efivarfs data for this variable
        let filename = format!("{}/{}/data", EFIVARFS_ROOT, name);

        let mut f = File::open(filename).map_err(|error| Error::for_variable(error, name))?;

        // Read variable contents
        let mut value: Vec<u8> = vec![];
        f.read_to_end(&mut value)
            .map_err(|error| Error::for_variable(error, name))?;

        Ok((value, flags))
    }
}

impl VarWriter for SystemManager {
    fn write(
        &mut self,
        name: &VariableName,
        attributes: VariableFlags,
        value: &[u8],
    ) -> crate::Result<()> {
        // Path to the attributes file
        let attributes_filename = format!("{}/{}/attributes", EFIVARFS_ROOT, name);
        // Open attributes file
        let mut f =
            File::open(attributes_filename).map_err(|error| Error::for_variable(error, name))?;
        let mut writer = BufWriter::new(&mut f);

        // Write attributes
        writer
            .write_all(attributes.to_string().as_bytes())
            .map_err(|error| Error::for_variable(error, name))?;

        // Filename to the matching efivarfs file for this variable
        let filename = format!("{}/{}/data", EFIVARFS_ROOT, name);

        let mut f = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(filename)
            .map_err(|error| Error::for_variable(error, name))?;

        // Write variable contents
        f.write(value)
            .map_err(|error| Error::for_variable(error, name))?;

        Ok(())
    }

    fn delete(&mut self, _name: &VariableName) -> crate::Result<()> {
        // Unimplemented because I wasn't able to enable efivars sysfs on my system
        unimplemented!("Variable deletion not supported on efivarfs. See https://github.com/iTrooz/efiboot-rs/issues/55");
    }
}

impl VarManager for SystemManager {}
