//! DLSS Neural Rendering architecture-gate patcher.
//!
//! Genuine `nvngx_dlssnr.dll` refuses to initialize feature 18 unless the GPU is
//! Blackwell (RTX 50). Two edits in the DLL's shared `.text` neutralize that CPU
//! -side check. This module applies them by byte-signature search, so it does not
//! depend on a fixed file offset or version.
//!
//! # Important: what this does and does not do
//!
//! The gate patch only bypasses the **CPU architecture check**. The neural
//! network kernels inside the DLL are compiled device code (cubins) built for a
//! specific GPU architecture. Patching the gate on a DLL whose cubins are for a
//! *different* architecture than the GPU you run on lets `CreateFeature` begin,
//! then fails at kernel load (`vkCreateCuModuleNVX ... failed`). Empirically
//! verified: a genuine Blackwell DLL, gate-patched, still cannot run on Ada.
//!
//! So this patcher is only useful on a DLL that already contains cubins for the
//! architecture you will run it on. It reproduces the exact edits the community
//! "Ada Lovelace+" build uses; it cannot synthesize kernels for an architecture
//! the DLL was not built for. For a working setup, prefer selecting a pre-built
//! per-architecture DLL (see [`crate::ArchTier`]).

use std::path::Path;

/// A byte-signature edit: find `find` exactly once, replace with `replace`
/// (equal length).
struct Edit {
    name: &'static str,
    find: &'static [u8],
    replace: &'static [u8],
}

// Site A: `mov dword [rsp+0x3c], 0x1b0` (target arch Blackwell2) -> `0x190` (Ada).
// Anchored by the following `call` displacement so it does not also match the
// twin site that sets the same value in another code path.
const SITE_A: Edit = Edit {
    name: "arch-target",
    find: &[
        0x48, 0x8b, 0xd3, 0xc7, 0x44, 0x24, 0x3c, 0xb0, 0x01, 0x00, 0x00, 0xe8, 0xc3, 0x98, 0x04,
        0x00,
    ],
    replace: &[
        0x48, 0x8b, 0xd3, 0xc7, 0x44, 0x24, 0x3c, 0x90, 0x01, 0x00, 0x00, 0xe8, 0xc3, 0x98, 0x04,
        0x00,
    ],
};

// Site B: in the arch-dispatch switch, the Ada case (`mov esi,0x190; jmp +0x15`)
// becomes `xor esi,esi; nop; nop; nop; jmp +0x4a`, redirecting to the accept path.
const SITE_B: Edit = Edit {
    name: "arch-dispatch",
    find: &[0xeb, 0x1c, 0xbe, 0x90, 0x01, 0x00, 0x00, 0xeb, 0x15],
    replace: &[0xeb, 0x1c, 0x33, 0xf6, 0x90, 0x90, 0x90, 0xeb, 0x4a],
};

/// Outcome of a successful patch.
#[derive(Debug, Clone)]
pub struct PatchReport {
    /// File offset of each edit that was applied.
    pub offsets: Vec<(&'static str, usize)>,
}

/// Reasons a patch could not be applied.
#[derive(Debug)]
pub enum PatchError {
    /// A signature was not found (wrong/corrupt DLL, or an unsupported version).
    NotFound(&'static str),
    /// A signature matched more than once, so the target is ambiguous.
    Ambiguous(&'static str, usize),
    /// The bytes already contain the patched form (idempotent no-op guard).
    AlreadyPatched,
}

impl std::fmt::Display for PatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatchError::NotFound(s) => write!(
                f,
                "patch site '{s}' not found (unsupported or non-genuine DLL)"
            ),
            PatchError::Ambiguous(s, n) => {
                write!(f, "patch site '{s}' matched {n} times (ambiguous)")
            }
            PatchError::AlreadyPatched => write!(f, "DLL is already gate-patched"),
        }
    }
}
impl std::error::Error for PatchError {}

fn find_unique(hay: &[u8], needle: &[u8]) -> Result<Option<usize>, usize> {
    let mut found = None;
    let mut count = 0;
    let mut i = 0;
    while i + needle.len() <= hay.len() {
        if &hay[i..i + needle.len()] == needle {
            count += 1;
            found = Some(i);
            i += needle.len();
        } else {
            i += 1;
        }
    }
    if count > 1 {
        return Err(count);
    }
    Ok(found)
}

/// Apply the Ada architecture-gate patch to a DLL image in memory.
///
/// Returns the offsets patched, or an error if the DLL is not the expected
/// genuine build. Idempotent: a already-patched image returns
/// [`PatchError::AlreadyPatched`] without modification.
pub fn patch_gate(bytes: &mut [u8]) -> Result<PatchReport, PatchError> {
    // Idempotency / already-patched check: if both replacements are already
    // present and neither original is, treat as already patched.
    let a_done = find_unique(bytes, SITE_A.replace).ok().flatten().is_some();
    let b_done = find_unique(bytes, SITE_B.replace).ok().flatten().is_some();
    let a_orig = find_unique(bytes, SITE_A.find);
    let b_orig = find_unique(bytes, SITE_B.find);
    if a_done && b_done && matches!(a_orig, Ok(None)) && matches!(b_orig, Ok(None)) {
        return Err(PatchError::AlreadyPatched);
    }

    let mut offsets = Vec::new();
    for edit in [&SITE_A, &SITE_B] {
        match find_unique(bytes, edit.find) {
            Ok(Some(off)) => {
                bytes[off..off + edit.replace.len()].copy_from_slice(edit.replace);
                offsets.push((edit.name, off));
            }
            Ok(None) => return Err(PatchError::NotFound(edit.name)),
            Err(n) => return Err(PatchError::Ambiguous(edit.name, n)),
        }
    }
    Ok(PatchReport { offsets })
}

/// True if the DLL image already carries the gate patch.
pub fn is_patched(bytes: &[u8]) -> bool {
    find_unique(bytes, SITE_A.find).ok().flatten().is_none()
        && find_unique(bytes, SITE_A.replace).ok().flatten().is_some()
}

/// Read a genuine DLL, apply the gate patch, and write the result to `dst`.
pub fn patch_file(src: &Path, dst: &Path) -> Result<PatchReport, Box<dyn std::error::Error>> {
    let mut bytes = std::fs::read(src)?;
    let report = patch_gate(&mut bytes)?;
    std::fs::write(dst, &bytes)?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patches_synthetic_image() {
        // Embed both original sites in a buffer of filler and confirm the edits
        // land and are idempotent afterwards.
        let mut buf = vec![0u8; 64];
        buf.extend_from_slice(SITE_A.find);
        buf.extend_from_slice(&[0u8; 32]);
        buf.extend_from_slice(SITE_B.find);
        buf.extend_from_slice(&[0u8; 16]);

        let report = patch_gate(&mut buf).expect("should patch");
        assert_eq!(report.offsets.len(), 2);
        assert!(is_patched(&buf));
        // Second run is a no-op.
        assert!(matches!(
            patch_gate(&mut buf),
            Err(PatchError::AlreadyPatched)
        ));
    }

    #[test]
    fn rejects_unrelated_bytes() {
        let mut buf = vec![0x90u8; 4096];
        assert!(matches!(patch_gate(&mut buf), Err(PatchError::NotFound(_))));
    }
}
