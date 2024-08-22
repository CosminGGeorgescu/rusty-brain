// * https://www.brainproducts.com/download/specification-of-brainvision-core-data-format-1-0/

use core::{f32, str};
use std::{fs, io::Read, path::Path, str::Split, u32};

use ndarray::{Array2, ArrayView1};

use super::BIDSPath;

mod locked {
    pub(crate) trait Locked {}

    impl Locked for f32 {}
    impl Locked for i16 {}
}

pub struct Header {
    data_file: String,
    marker_file: String,
    num_channels: u32,
    sampling_interval: f64,
    averaged: bool,
    averaged_segms: u32,
    num_data_points: u32,
    segmentation_type: String,
    binary_format: BinaryFormatType,
    channels: Vec<ChannelInfo>,
    channel_coords: Option<Vec<Coordinates>>,
    comment: Option<String>,
}

impl Header {
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
        let num_data_points = if averaged && segmentation_type == "MARKERBASED" {
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
            num_data_points,
            segmentation_type,
            binary_format,
            channels,
            channel_coords,
            comment,
        }
    }

    pub fn data_file(&self) -> &str {
        &self.data_file
    }

    pub fn num_channels(&self) -> u32 {
        self.num_channels
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

#[derive(Clone, Copy)]
pub enum BinaryFormatType {
    IeeeFloat32,
    Int16,
}

struct ChannelInfo {
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

struct Coordinates {
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

#[allow(private_bounds)]
pub struct Data<T: BinaryFormat> {
    data: Array2<T>,
}

#[allow(private_bounds)]
impl<T: BinaryFormat> Data<T> {
    pub fn load<P: AsRef<Path>>(path: &BIDSPath<P>, header: &Header) -> Data<T> {
        let rawdata = fs::read(path.path.join(header.data_file())).unwrap();
        let num_channels = header.num_channels() as usize;
        let chunks = rawdata.chunks_exact(T::BYTES);
        // ! Actual number of data points is `effective_data_points` + 1
        let effective_data_points = chunks.len() / num_channels;
        let data = Array2::from_shape_vec(
            (num_channels, effective_data_points),
            chunks.map(|c| T::from_bytes(c)).collect::<Vec<T>>(),
        )
        .unwrap();

        Data { data }
    }

    pub fn channel(&self, index: usize) -> ArrayView1<T> {
        self.data.column(index)
    }
}
