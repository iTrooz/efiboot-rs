//! Definition of the Variable type

use std::fmt;
use std::str::FromStr;

use super::EFI_GUID;
use crate::Error;

#[derive(Copy, Clone, Eq)]
/// An EFI variable vendor identifier
pub enum VariableVendor {
    /// Standard EFI variables
    Efi,
    /// Other EFI vendors
    Custom(uuid::Uuid),
}

impl VariableVendor {
    /// Return true if this vendor is the EFI vendor
    pub fn is_efi(&self) -> bool {
        matches!(self, VariableVendor::Efi)
    }
}

impl PartialEq for VariableVendor {
    fn eq(&self, other: &Self) -> bool {
        match self {
            VariableVendor::Efi => match other {
                VariableVendor::Efi => true,
                VariableVendor::Custom(uuid) => *uuid == *EFI_GUID,
            },
            VariableVendor::Custom(uuid) => match other {
                VariableVendor::Efi => *uuid == *EFI_GUID,
                VariableVendor::Custom(other_uuid) => *other_uuid == *uuid,
            },
        }
    }
}

impl From<uuid::Uuid> for VariableVendor {
    fn from(other: uuid::Uuid) -> Self {
        if other == *EFI_GUID {
            VariableVendor::Efi
        } else {
            VariableVendor::Custom(other)
        }
    }
}

impl AsRef<uuid::Uuid> for VariableVendor {
    fn as_ref(&self) -> &uuid::Uuid {
        match self {
            VariableVendor::Efi => &EFI_GUID,
            VariableVendor::Custom(uuid) => uuid,
        }
    }
}

impl fmt::Debug for VariableVendor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.as_ref(), f)
    }
}

impl fmt::Display for VariableVendor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_ref(), f)
    }
}

/// An EFI variable name
///
/// # Examples
///
/// Parsing a valid name into succeeds:
///
/// ```
/// # use std::str::FromStr;
/// # use efivar::efi::Variable;
/// let name = Variable::from_str("BootOrder-8be4df61-93ca-11d2-aa0d-00e098032b8c").unwrap();
/// assert_eq!(*name.vendor().as_ref(), uuid::Uuid::from_str("8be4df61-93ca-11d2-aa0d-00e098032b8c").unwrap());
/// assert_eq!(name.variable(), "BootOrder");
/// ```
///
/// Parsing an invalid name fails:
///
/// ```
/// # use std::str::FromStr;
/// # use efivar::efi::Variable;
/// let result = Variable::from_str("invalid name");
/// assert!(result.is_err());
/// ```
///
/// Turning the structure back into a string:
///
/// ```
/// # use efivar::efi::Variable;
/// let name = Variable::new("BootOrder");
/// assert_eq!(name.to_string(), "BootOrder-8be4df61-93ca-11d2-aa0d-00e098032b8c");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable {
    /// Variable name
    variable: String,
    /// Vendor identifier
    vendor: VariableVendor,
}

impl Variable {
    /// Create a new EFI standard variable name
    ///
    /// # Parameters
    ///
    /// * `variable`: name of the variable
    pub fn new(variable: &str) -> Self {
        Self {
            variable: variable.to_owned(),
            vendor: VariableVendor::Efi,
        }
    }

    /// Create a new custom vendor variable name
    ///
    /// # Parameters
    ///
    /// * `variable`: name of the variable
    /// * `vendor`: vendor identifier
    pub fn new_with_vendor<V: Into<VariableVendor>>(variable: &str, vendor: V) -> Self {
        Self {
            variable: variable.to_owned(),
            vendor: vendor.into(),
        }
    }

    /// Get the variable name for this instance
    pub fn variable(&self) -> &str {
        &self.variable
    }

    /// Get the vendor for this instance
    pub fn vendor(&self) -> &VariableVendor {
        &self.vendor
    }

    /// Return a short version of the variable name as a String
    ///
    /// If the vendor GUID is the EFI one, it will not be added to the name.
    pub fn short_name(&self) -> String {
        if self.vendor.is_efi() {
            self.variable.clone()
        } else {
            self.to_string()
        }
    }

    /// Returns the boot var ID (4 digits hex number) if this variable is a boot entry. Else, return None
    pub fn boot_var_id(&self) -> Option<u16> {
        if self.variable.len() == 8 && &self.variable[0..4] == "Boot" {
            self.variable[4..8].parse().ok()
        } else {
            None
        }
    }
}

impl FromStr for Variable {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let name_parts = s.splitn(2, '-').collect::<Vec<_>>();
        if name_parts.len() != 2 {
            return Err(Error::InvalidVarName { name: s.into() });
        }

        // Parse GUID
        let vendor = uuid::Uuid::from_str(name_parts[1])
            .map_err(|error| crate::Error::UuidError { error })?;

        Ok(Self::new_with_vendor(name_parts[0], vendor))
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}", self.variable, self.vendor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_name_invalid() {
        assert!(Variable::from_str("BootOrder_Invalid").is_err());
    }

    #[test]
    fn to_fullname_partial() {
        assert_eq!(
            Variable::new("BootOrder").to_string(),
            "BootOrder-8be4df61-93ca-11d2-aa0d-00e098032b8c"
        );
    }
}
