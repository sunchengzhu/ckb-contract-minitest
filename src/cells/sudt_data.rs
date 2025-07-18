use crate::cell_message::cell::MoleculeStructFlag;
use serde::{Deserialize, Serialize};
use crate::impl_cell_methods;

#[derive(PartialEq, Debug)]
pub struct SUDTDataCell {
    pub lock_arg: u8,
    pub type_arg: Option<[u8; 32]>,
    pub data: SUDTData,
    pub witness: Option<()>,  // For placeholder only, not actually used
    pub struct_flag: MoleculeStructFlag,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct SUDTData {
    pub amount: u128,
}

impl SUDTDataCell {
    pub(crate) fn default() -> Self {
        SUDTDataCell {
            lock_arg: 0,
            type_arg: None,
            data: SUDTData { amount: 0 },
            witness: None,
            struct_flag: MoleculeStructFlag {
                lock_arg: true,
                type_arg: true,
                data: true,
                witness: false, // SUDT does not use witness
            },
        }
    }

    pub fn new(type_arg: [u8; 32], data: SUDTData) -> Self {
        SUDTDataCell {
            lock_arg: 0,
            type_arg: Some(type_arg),
            data,
            witness: None,
            struct_flag: MoleculeStructFlag {
                lock_arg: true,
                type_arg: true,
                data: true,
                witness: false, // SUDT does not use witness
            },
        }
    }
}

impl_cell_methods!(SUDTDataCell);
