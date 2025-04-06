// * https://www.brainproducts.com/download/specification-of-brainvision-core-data-format-1-0/

use core::{f32, str};
use std::{fmt::Debug, fs, io::Read, path::Path, str::Split};

use ndarray::{Array2, ArrayView1};

use super::BIDSPath;

mod locked {
    // Trait used to restrict the data type the raw data can be formatted to
    // BrainVision Core Data Format 1.0 supports only `f32` and `i16`
    pub(crate) trait Locked {}

    impl Locked for f32 {}
    impl Locked for i16 {}
}

// Struct containing all of the information provided in the header file of the associated `task`,
// `acquisition` and `run`, provided to the `Header::load` method.
//
// sub-<subject>[_ses-<session>]_task-<task>[_acq-<acquisition>][_run-<run>]_eeg.vhdr
#[derive(Debug)]
pub struct Header {
    // Name of the EEG data file
    pub data_file: String,
    // Name of marker file
    pub marker_file: String,
    // Number of channels in the EEG data file
    pub num_channels: u32,
    pub sampling_interval: f64,
    // Indicates whether the data set is averaged across segments
    pub averaged: bool,
    // Number of segments included in the average
    pub averaged_segms: u32,
    // Number of samples per channel
    pub segment_data_points: u32,
    // Type of segmentation
    // - NOTSEGMENTED: The data set is not segmented
    // - MARKERBASED: The data set is segmented based
    pub segmentation_type: String,
    // Encoding of data in EEG data file
    // - IEEE_FLOAT_32: IEEE floating-point format, single precision, 4 bytes per value
    // - INT_16: 16-bit signed integer
    pub binary_format: BinaryFormatType,
    // Stores information about each channel, provided in the `[Channel Infos]` section
    pub channels: Vec<ChannelInfo>,
    // Stores information about each channel's coordinates, provided in the `Coordinates` section
    pub channel_coords: Option<Vec<Coordinates>>,
    // Stores the `[Comment]` section
    pub comment: Option<String>,
}

impl Header {
    // Load a header file by providing the `path` to a BIDS-compliant data recording
    pub fn load<P: AsRef<Path>>(
        path: &BIDSPath<P>,
        task: &str,
        acquisition: Option<&str>,
        run: Option<&str>,
    ) -> Header {
        let mut buf = String::new();
        let _ = fs::File::open(path.path.join(format!(
            "sub-{}{}_task-{}{}{}_{}.vhdr",
            path.subject,
            if let Some(session) = path.session {
                format!("_ses-{}", session)
            } else {
                String::default()
            },
            task,
            if let Some(acquisition) = acquisition {
                format!("_acq-{}", acquisition)
            } else {
                String::default()
            },
            if let Some(run) = run {
                format!("_run-{}", run)
            } else {
                String::default()
            },
            path.datatype
        )))
        .unwrap()
        .read_to_string(&mut buf);
        // Extract the `[Comment]` section
        let comment = buf
            .match_indices("[Comment]")
            .next()
            .map(|(idx, _)| buf[idx + "[Comment]".len()..buf.len()].to_string());
        // And skip the first line (identification line)
        buf = buf.lines().skip(1).collect::<Vec<&str>>().join("\n");

        let file = ini::Ini::load_from_str(&buf).unwrap();

        let common_infos = file.section(Some("Common Infos")).unwrap();
        let binary_infos = file.section(Some("Binary Infos")).unwrap();
        let channel_infos = file.section(Some("Channel Infos")).unwrap();
        let coordinates = file.section(Some("Coordinates"));

        let data_file = common_infos.get("DataFile").unwrap().into();
        let marker_file = common_infos.get("MarkerFile").unwrap().into();
        let num_channels = common_infos
            .get("NumberOfChannels")
            .map(|s| s.parse::<u32>().unwrap())
            .unwrap();
        let sampling_interval = common_infos
            .get("SamplingInterval")
            .map(|s| s.parse::<f64>().unwrap())
            .unwrap();
        let averaged = common_infos.get("Averaged").map_or_else(
            || false,
            |s| match s {
                "YES" => true,
                "NO" | _ => false,
            },
        );
        let averaged_segms = match averaged {
            false => 0u32,
            true => common_infos
                .get("AveragedSegments")
                .map(|s| s.parse::<u32>().unwrap())
                .unwrap(),
        };
        let segmentation_type = match averaged {
            false => "NOTSEGMENTED",
            true => common_infos.get("SegmentationType").unwrap(),
        }
        .into();
        let segment_data_points = if averaged && segmentation_type == "MARKERBASED" {
            common_infos
                .get("SegmentDataPoints")
                .map(|s| s.parse::<u32>().unwrap())
                .unwrap()
        } else {
            0
        };

        let binary_format = match binary_infos.get("BinaryFormat").unwrap() {
            "IEEE_FLOAT_32" => BinaryFormatType::IeeeFloat32,
            "INT_16" => BinaryFormatType::Int16,
            _ => panic!("Invalid binary format !"),
        };

        let channels = channel_infos
            .iter()
            .map(|(_, v)| ChannelInfo::from(v.split(',')))
            .collect::<Vec<ChannelInfo>>();

        let channel_coords = coordinates.map(|coords| {
            coords
                .iter()
                .map(|(_, v)| Coordinates::from(v.split(',')))
                .collect::<Vec<Coordinates>>()
        });

        Header {
            data_file,
            marker_file,
            num_channels,
            sampling_interval,
            averaged,
            averaged_segms,
            segment_data_points,
            segmentation_type,
            binary_format,
            channels,
            channel_coords,
            comment,
        }
    }
}

pub(crate) trait BinaryFormat: locked::Locked + Sized {
    const BYTES: usize;

    fn from_bytes(bytes: &[u8]) -> Self;
}

impl BinaryFormat for f32 {
    const BYTES: usize = 4;

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut rep = [0u8; 4];
        rep.copy_from_slice(bytes);
        f32::from_le_bytes(rep)
    }
}

impl BinaryFormat for i16 {
    const BYTES: usize = 2;

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut rep = [0u8; 2];
        rep.copy_from_slice(bytes);
        i16::from_le_bytes(rep)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum BinaryFormatType {
    IeeeFloat32,
    Int16,
}

// Information about a channel
#[derive(Debug)]
pub struct ChannelInfo {
    name: String,
    ref_name: String,
    resolution: f64,
    unit: String,
}

impl From<Split<'_, char>> for ChannelInfo {
    fn from(mut value: Split<char>) -> Self {
        let name = value.next().unwrap().into();
        let ref_name = value
            .next()
            .map(|s| if s.is_empty() { "Cz" } else { s })
            .unwrap()
            .into();
        let resolution = value
            .next()
            .map(|s| {
                if s.is_empty() {
                    1.0f64
                } else {
                    s.parse::<f64>().unwrap()
                }
            })
            .unwrap();
        let unit = value
            .next()
            .map(|s| if s.is_empty() { "μV" } else { s })
            .unwrap()
            .into();

        Self {
            name,
            ref_name,
            resolution,
            unit,
        }
    }
}

// Coordinates of a channel
#[derive(Debug)]
pub struct Coordinates {
    radius: f64,
    theta: f64,
    phi: f64,
}

impl From<Split<'_, char>> for Coordinates {
    fn from(mut value: Split<'_, char>) -> Self {
        Coordinates {
            radius: value.next().map(|s| s.parse::<f64>().unwrap()).unwrap(),
            theta: value.next().map(|s| s.parse::<f64>().unwrap()).unwrap(),
            phi: value.next().map(|s| s.parse::<f64>().unwrap()).unwrap(),
        }
    }
}

// The formated data associated with a header
//
// sub-<subject>[_ses-<session>]_task-<task>[_acq-<acquisition>][_run-<run>]_eeg.eeg
#[allow(private_bounds)]
#[derive(Debug)]
pub struct Data<T: BinaryFormat> {
    data: Array2<T>,
}

#[allow(private_bounds)]
impl<T: BinaryFormat + Clone> Data<T> {
    pub fn load<P: AsRef<Path>>(path: &BIDSPath<P>, header: &Header) -> Data<T> {
        // Load the raw data
        let rawdata = fs::read(path.path.join(header.data_file.as_str())).unwrap();
        let num_channels = header.num_channels as usize;
        // Cut the raw data into chunks based on the binary format of the dataset's data type
        let chunks = rawdata.chunks_exact(T::BYTES);
        // Actual number of data points is `effective_data_points` + 1
        let effective_data_points = chunks.len() / num_channels;
        // Format the raw data according to the binary representation
        // Data orientation is N x M, where N is the number of channels and M is number of samples
        let data = Array2::from_shape_vec(
            (effective_data_points, num_channels),
            chunks.map(|c| T::from_bytes(c)).collect::<Vec<T>>(),
        )
        .unwrap()
        .t()
        .to_owned();

        Data { data }
    }

    pub fn channel(&self, index: usize) -> ArrayView1<T> {
        self.data.row(index)
    }
}
