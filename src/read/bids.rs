pub mod brainvision_core {
    // * https://www.brainproducts.com/download/specification-of-brainvision-core-data-format-1-0/

    use std::{path::Path, str::Split};

    pub(crate) struct Header {
        data_file: String,
        marker_file: String,
        num_channels: u32,
        sampling_interval: f64,
        averaged: bool,
        averaged_segms: u32,
        num_data_points: u32,
        segmentation_type: String,
        binary_format: BinaryFormat,
        channels: Vec<Channel>,
        channel_coords: Option<Vec<Coordinates>>,
        // TODO: Find a to load the [Comment] section as an entire String
        comment: Option<String>,
    }

    impl Header {
        pub fn load_header<P>(path: P) -> Header
        where
            P: AsRef<Path>,
        {
            // TODO: Strip the first line of the file
            let file = ini::Ini::load_from_file(path).unwrap();

            let common_infos = file.section(Some("Common Infos")).unwrap();
            let binary_infos = file.section(Some("Binary Infos")).unwrap();
            let channel_infos = file.section(Some("Channel Infos")).unwrap();
            let coordinates = file.section(Some("Coordinates"));
            let comment = file.section(Some("Comment"));

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
                "IEEE_FLOAT_32" => BinaryFormat::IeeeFloat32,
                "INT_16" => BinaryFormat::Int16,
                _ => panic!("Invalid binary format !"),
            };

            let channels = channel_infos
                .iter()
                .map(|(_, v)| Channel::from(v.split(',')))
                .collect::<Vec<Channel>>();

            let channel_coords = match coordinates {
                Some(coords) => Some(
                    coords
                        .iter()
                        .map(|(_, v)| Coordinates::from(v.split(',')))
                        .collect::<Vec<Coordinates>>(),
                ),
                None => None,
            };

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
                comment: None,
            }
        }
    }

    enum BinaryFormat {
        IeeeFloat32 = 0,
        Int16 = 1,
    }

    #[derive(Debug)]
    struct Channel {
        name: String,
        ref_name: String,
        resolution: f64,
        unit: String,
    }

    impl From<Split<'_, char>> for Channel {
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

    impl Coordinates {
        pub(crate) const fn new(radius: f64, theta: f64, phi: f64) -> Self {
            Self { radius, theta, phi }
        }
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
}
