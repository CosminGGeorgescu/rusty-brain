use std::path::{Path, PathBuf};

pub mod brainvision_core;

// The path to a BIDS-compliant data recording root
// Each recording is uniquely identified by a `root`, `subject`, `session` and a type of data recorded
// and points to a directory:
//
// sub-<subject>/
//     -> [ses-<ses>]/
//         -> eeg/
//
// As of now, it is adapted specifically for the EEG specification
// <https://bids-specification.readthedocs.io/en/stable/modality-specific-files/electroencephalography.html>
pub struct BIDSPath<'a, P: AsRef<Path>> {
    path: PathBuf,
    root: P,
    subject: &'a str,
    session: Option<&'a str>,
    datatype: &'a str,
}

impl<'a, P: AsRef<Path>> BIDSPath<'a, P> {
    pub fn new(root: P, subject: &'a str, session: Option<&'a str>, datatype: &'a str) -> Self {
        let mut path = root.as_ref().join(format!("sub-{}", subject));
        if let Some(session) = session {
            path.push(format!("ses-{}", session));
        }
        path.push(datatype);

        Self {
            path,
            root,
            subject,
            session,
            datatype,
        }
    }
}
