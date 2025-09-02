use crate::elf::{ElfFile, LoadSegment, PrStatus, Relocation};
use std::error::Error;

pub struct ElfFiles {
    pub core: Option<ElfFile>,
    pub exe: Option<ElfFile>,
}

impl ElfFiles {
    pub fn new(paths: Vec<std::path::PathBuf>) -> Result<Self, Box<dyn Error>> {
        let files = paths
            .into_iter()
            .map(|p| ElfFile::new(p))
            .collect::<Result<Vec<_>, _>>()?;
        let mut core = None;
        let mut exe = None;
        for file in files {
            if file.header.etype == 4 {
                if core.is_none() {
                    core = Some(file);
                } else {
                    return Err("can't have multiple core files".into());
                }
            } else if exe.is_none() {
                exe = Some(file);
            } else {
                return Err("can't have multiple exe files".into());
            }
        }
        Ok(ElfFiles { core, exe })
    }

    pub fn find_load_segment(&self, vaddr: u64) -> Option<&LoadSegment> {
        match &self.core {
            Some(c) => c.find_load_segment(vaddr),
            None => None,
        }
    }

    pub fn find_prstatus(&self) -> Option<PrStatus> {
        match &self.core {
            Some(c) => c.find_prstatus(),
            None => None,
        }
    }

    pub fn find_vaddr(&self, offset: u64) -> Option<(&LoadSegment, u64)> {
        match &self.core {
            Some(c) => c.find_vaddr(offset),
            None => None,
        }
    }

    pub fn find_relocations(&self) -> Vec<Relocation> {
        let mut result = Vec::new();
        if let Some(file) = &self.core {
            file.find_relocations(&mut result);
        }
        if let Some(file) = &self.exe {
            file.find_relocations(&mut result);
        }
        result
    }
}
