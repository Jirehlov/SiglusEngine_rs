//! Siglus scene-script core (PCK/DAT parsing + disassembler).

pub mod angou;
pub mod angou_consts;
pub mod dat;
pub mod elm;
pub mod gameexe;
pub mod lzss;
pub mod pck;
pub mod resource;

// VM / interpreter (work in progress)
pub mod lexer;
pub mod stack;
pub mod vm;

// Higher-level runtime that can load .pck and run a scene.
pub mod runtime;
