use crate::{
    debug::LineInfo,
    elf::{ElfFile, LoadSegment, PrStatus, Relocation, VirtualAddr},
};
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

    pub fn find_load_segment(&self, vaddr: VirtualAddr) -> Option<&LoadSegment> {
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

    // pub fn find_vaddr(&self, offset: u64) -> Option<(&LoadSegment, u64)> {
    //     match &self.core {
    //         Some(c) => c.find_vaddr(offset),
    //         None => None,
    //     }
    // }

    /// Returns file name, line number, and column for the given address.
    pub fn find_line(&self, addr: VirtualAddr) -> Result<(String, u32, u16), Box<dyn Error>> {
        match (&self.core, &self.exe) {
            (Some(core), Some(exe)) => {
                match core.vaddr_to_raddr(addr) {
                    Some(addr) => {
                        match exe.get_lines() {
                            // TODO need to cache lines
                            Some(lines) => match lines.lines.get(&addr) {
                                Some(value) => {
                                    let file = lines.files.get(value.file);
                                    Ok((file.clone(), value.line, value.column))
                                }
                                None => Ok(("?".to_string(), 0, 0)),
                            },
                            None => Err("Couldn't find .debug_line section".into()),
                        }
                    }
                    None => Err("couldn't find a load segment matching the addr".into()),
                }
            }
            (None, Some(_)) => Err("need an core file to find file and line".into()),
            (Some(_), None) => Err("need an exe file to find file and line".into()), // TODO addr2line doesn't need a core file
            (None, None) => Err("need core and exe files to find file and line".into()),
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
